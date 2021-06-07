use core::fmt::{Display, Formatter, Result as FmtResult};
use std::cell::RefCell;
use std::convert::TryInto;
use std::error::Error as StdError;
use std::thread::sleep;
use std::time::Duration;

use crate::skia::gpu::gl::{Format as SkiaGLFormat, FramebufferInfo};
use crate::skia::gpu::{
    BackendRenderTarget, DirectContext as SkiaDirectContext, SurfaceOrigin,
};
use crate::skia::{ColorType, Point, Surface};

use glutin::dpi::LogicalSize;
use glutin::event::{Event, MouseButton, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::monitor::VideoMode;
use glutin::window::{Fullscreen, Window, WindowBuilder};
use glutin::{
    ContextError as GLContextError, ContextWrapper as GlutinContextWrapper,
    CreationError as GLCreationError, GlProfile,
    PossiblyCurrent as GlutinPossiblyCurrent,
};

use gl::types::GLint;
use gl_rs as gl;

use super::input::{EventHandleResult, InputState};
use super::time::TimeState;
use super::Game;

type WindowedContext = GlutinContextWrapper<GlutinPossiblyCurrent, Window>;

#[derive(Debug)]
pub enum GameError {
    RunnerError(RunnerError),
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            GameError::RunnerError(s) => {
                write!(f, "runner error: {}", s)
            }
        }
    }
}

impl StdError for GameError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            GameError::RunnerError(e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum RunnerError {
    GLCreationError(GLCreationError),
    GLContextError(GLContextError),
}

impl Display for RunnerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            RunnerError::GLCreationError(s) => {
                write!(f, "OpenGL window creation error: {}", s)
            }
            RunnerError::GLContextError(s) => {
                write!(f, "OpenGL context manipulation error: {}", s)
            }
        }
    }
}

impl StdError for RunnerError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            RunnerError::GLCreationError(e) => Some(e),
            RunnerError::GLContextError(e) => Some(e),
        }
    }
}

impl From<GLCreationError> for RunnerError {
    fn from(e: GLCreationError) -> Self {
        Self::GLCreationError(e)
    }
}

impl From<GLContextError> for RunnerError {
    fn from(e: GLContextError) -> Self {
        Self::GLContextError(e)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct ID(u64);

impl ID {
    pub fn next() -> Self {
        Self(State::with_mut(|x| {
            let id = x.id_keeper;
            x.id_keeper += 1;
            id
        }))
    }
}

pub struct State {
    pub input_state: InputState,
    pub time_state: TimeState,
    pub time_state_draw: TimeState,

    was_fullscreen: bool,
    is_fullscreen: bool,

    id_keeper: u64,
}

impl State {
    const PANIC_MESSAGE: &'static str =
        "Attempt to get game state while game is uninitialized";
    thread_local!(static STATE: RefCell<Option<State>> = RefCell::new(None));

    #[inline]
    #[track_caller]
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        Self::STATE.with(|x| f(x.borrow().as_ref().expect(Self::PANIC_MESSAGE)))
    }

    #[inline]
    #[track_caller]
    fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        Self::STATE
            .with(|x| f(x.borrow_mut().as_mut().expect(Self::PANIC_MESSAGE)))
    }

    pub fn last_update_time() -> Duration {
        Self::with(|x| x.time_state.last_update_time())
    }

    pub fn last_update_time_draw() -> Duration {
        Self::with(|x| x.time_state_draw.last_update_time())
    }

    pub fn elapsed() -> Duration {
        Self::with(|x| x.time_state.elapsed())
    }

    pub fn elapsed_draw() -> Duration {
        Self::with(|x| x.time_state_draw.elapsed())
    }

    pub fn is_fullscreen() -> bool {
        Self::with(|x| x.is_fullscreen)
    }

    pub fn set_fullscreen(fullscreen: bool) {
        Self::with_mut(|x| x.is_fullscreen = fullscreen);
    }

    pub fn toggle_fullscreen() -> bool {
        Self::with_mut(|x| {
            x.is_fullscreen = !x.is_fullscreen;
            x.is_fullscreen
        })
    }

    pub fn mouse_position() -> Point {
        Self::with(|x| x.input_state.mouse_position())
    }

    pub fn is_key_down(key: VirtualKeyCode) -> bool {
        Self::with(|x| x.input_state.is_key_down(key))
    }

    pub fn is_mouse_down(button: MouseButton) -> bool {
        Self::with(|x| x.input_state.is_mouse_down(button))
    }
}

pub fn run<F, T, E>(
    game: F,
    size: LogicalSize<f64>,
    title: &str,
) -> Result<E, RunnerError>
where
    F: FnOnce() -> Result<T, E>,
    T: Game + 'static,
{
    let (event_loop, win_ctx) = init_runner(size, title)?;

    let fb_info = create_fb_info();
    let mut gr_ctx = SkiaDirectContext::new_gl(None, None).unwrap();
    let mut surface = create_surface(&win_ctx, fb_info, &mut gr_ctx);

    init_state(win_ctx.window());

    let mut game = match game() {
        Ok(e) => e,
        Err(e) => return Ok(e),
    };
    game.set_size(State::with(|x| x.input_state.window_size));

    let target_update_time = Duration::from_millis(1); // 1000 fps

    event_loop.run(move |event, _, flow| {
        // https://github.com/rust-skia/rust-skia/issues/476
        // The following 3 lines of code fixes the entire application hanging
        // on Windows when you `*flow = ControlFlow::Exit`.
        // This is like the TF2 coconut texture, but this time it's real.
        // I love the software industry.
        // As per GitHub user `kognise`:
        //
        // "Skia will hang - the program doesn't exit and cpu usage goes to
        // around 30%. [...] This only happens if:
        // - It's being run on Windows
        // - The window handle, the graphics context, AND the Skia surface are
        //   moved into the closure
        // They AREN'T accessed in the order of surface, then context, then
        // handle"
        //
        // ...hence the ordering of the 3 lines of code.
        // Please do not touch this whatever you do with the code around here,
        // for it holds together the fabric of space and time... at least on
        // Windows.
        let surface = &mut surface;
        let gr_ctx = &mut gr_ctx;
        let win_ctx = &win_ctx;

        match event {
            Event::WindowEvent { event, .. } => {
                if let WindowEvent::Resized(size) = &event {
                    *surface = create_surface(win_ctx, fb_info, gr_ctx);
                    win_ctx.resize(*size);
                }
                if game_handle_event(&mut game, event) {
                    game.close();
                    *flow = ControlFlow::Exit;
                }
            }
            Event::MainEventsCleared => {
                game.update();

                if let Some(s) = State::with_mut(|x| {
                    if x.is_fullscreen != x.was_fullscreen {
                        x.was_fullscreen = x.is_fullscreen;
                        Some(x.is_fullscreen)
                    } else {
                        None
                    }
                }) {
                    set_fullscreen(s, win_ctx.window());
                }

                win_ctx.window().request_redraw();

                State::with_mut(|state| {
                    let last_update = state.time_state.last_update();
                    let last_update_time = last_update.elapsed();
                    if last_update_time < target_update_time {
                        sleep(target_update_time - last_update_time);
                    }

                    state.time_state.update();
                });
            }
            Event::RedrawRequested(_) => {
                State::with_mut(|state| state.time_state_draw.update());
                let canvas = surface.canvas();
                let sf = win_ctx.window().scale_factor() as f32;
                canvas.reset_matrix();
                canvas.scale((sf, sf));
                game.draw(canvas);
                gr_ctx.flush(None);
                if let Err(e) = win_ctx.swap_buffers() {
                    game.crash(GameError::RunnerError(e.into()));
                    game.close();
                    *flow = ControlFlow::Exit;
                }
            }
            _ => {}
        }
    });
}

fn set_fullscreen(enable: bool, win: &Window) {
    win.set_fullscreen(if enable {
        default_video_mode(win).map(|e| Fullscreen::Exclusive(e))
    } else {
        None
    });
}

fn default_video_mode(win: &Window) -> Option<VideoMode> {
    win.current_monitor()?.video_modes().next()
}

fn game_handle_event(game: &mut impl Game, event: WindowEvent) -> bool {
    if let Some(r) = State::with_mut(|x| x.input_state.handle_event(event)) {
        match r {
            EventHandleResult::Input(event) => game.input(event),
            EventHandleResult::Resized(size) => {
                game.set_size(size);
            }
            EventHandleResult::Exit => {
                return true;
            }
        }
    }

    false
}

fn init_runner(
    size: LogicalSize<f64>,
    title: &str,
) -> Result<(EventLoop<()>, WindowedContext), RunnerError> {
    let event_loop = EventLoop::new();
    let win = WindowBuilder::new().with_inner_size(size).with_title(title);
    let ctx_builder = glutin::ContextBuilder::new()
        .with_vsync(false)
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_double_buffer(Some(true))
        .with_gl_profile(GlProfile::Core);

    let win_ctx = ctx_builder.build_windowed(win, &event_loop)?;
    let win_ctx = unsafe { win_ctx.make_current().map_err(|e| e.1)? };

    gl::load_with(|s| win_ctx.get_proc_address(s));

    Ok((event_loop, win_ctx))
}

fn init_state(win: &Window) {
    let input_state = InputState::new(win.inner_size(), win.scale_factor());
    let time_state = TimeState::new();
    let time_state_draw = TimeState::new();
    State::STATE.with(|x| {
        *x.borrow_mut() = Some(State {
            input_state,
            time_state,
            time_state_draw,
            was_fullscreen: false,
            is_fullscreen: false,
            id_keeper: 0,
        });
    });
}

fn create_fb_info() -> FramebufferInfo {
    let mut fboid: GLint = 0;
    unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };
    FramebufferInfo {
        fboid: fboid.try_into().unwrap(),
        format: SkiaGLFormat::RGBA8.into(),
    }
}

fn create_surface(
    win_ctx: &WindowedContext,
    fb_info: FramebufferInfo,
    gr_ctx: &mut SkiaDirectContext,
) -> Surface {
    let pix = win_ctx.get_pixel_format();
    let size = win_ctx.window().inner_size();
    let size = (
        size.width.try_into().unwrap(),
        size.height.try_into().unwrap(),
    );
    let target = BackendRenderTarget::new_gl(
        size,
        pix.multisampling.map(|s| s.try_into().unwrap()),
        pix.stencil_bits.try_into().unwrap(),
        fb_info,
    );
    Surface::from_backend_render_target(
        gr_ctx,
        &target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
    .unwrap()
}
