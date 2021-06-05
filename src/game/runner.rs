use core::fmt::{Display, Formatter, Result as FmtResult};
use std::cell::RefCell;
use std::error::Error as StdError;
use std::thread::sleep;
use std::time::Duration;

use crate::skia::{Color, Point, Size};

use super::input::{EventHandleResult, InputState};
use super::time::TimeState;
use super::Game;

use sdl2::{
    event::Event as Sdl2Event,
    keyboard::Keycode,
    mouse::MouseButton,
    video::{FullscreenType, Window as Sdl2Window},
};
use skulpin_renderer::rafx::api::{RafxError, RafxExtents2D};
use skulpin_renderer::{LogicalSize, RendererBuilder};

#[derive(Debug)]
pub enum Error {
    RendererError(RafxError),
    FullscreenError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::RendererError(e) => e.fmt(f),
            Error::FullscreenError(s) => write!(f, "{}", s),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::RendererError(e) => Some(e),
            Error::FullscreenError(_) => None,
        }
    }
}

impl From<RafxError> for Error {
    fn from(result: RafxError) -> Self {
        Error::RendererError(result)
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
    fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&Self) -> R,
    {
        Self::STATE.with(|x| f(x.borrow().as_ref().expect(Self::PANIC_MESSAGE)))
    }

    #[inline]
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

    pub fn is_key_down(key: Keycode) -> bool {
        Self::with(|x| x.input_state.is_key_down(key))
    }

    pub fn is_mouse_down(button: MouseButton) -> bool {
        Self::with(|x| x.input_state.is_mouse_down(button))
    }
}

pub struct Runner;

impl Runner {
    const BACKGROUND: Color = Color::from_argb(255, 10, 10, 10);

    pub fn run<F, T, E>(
        game: F,
        size: LogicalSize,
        title: &str,
        renderer_builder: RendererBuilder,
    ) -> Result<(), E>
    where
        F: FnOnce() -> Result<T, E>,
        T: Game,
    {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let sdl_video = sdl.video().expect("Failed to initialize SDL2 video");

        let mut win = sdl_video
            .window(title, size.width, size.height)
            .allow_highdpi()
            .resizable()
            .build()
            .expect("Failed to create game window");

        let (width, height) = win.vulkan_drawable_size();
        let extents = RafxExtents2D { width, height };

        let mut renderer = renderer_builder
            .build(&win, extents)
            .expect("Failed to create renderer");

        let mut event_pump =
            sdl.event_pump().expect("Failed to create SDL2 event pump");

        let target_update_time = Duration::from_millis(1); // 1000 fps

        let input_state = InputState::new(size);
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

        let mut game = game()?;
        game.set_size(
            State::STATE
                .with(|x| x.borrow().as_ref().unwrap().input_state.window_size),
        );

        'events: loop {
            game.update();

            for event in event_pump.poll_iter() {
                if Self::game_handle_event(&mut game, event) {
                    break 'events;
                }
            }

            if let Some(s) = State::with_mut(|x| {
                if x.is_fullscreen != x.was_fullscreen {
                    x.was_fullscreen = x.is_fullscreen;
                    Some(x.is_fullscreen)
                } else {
                    None
                }
            }) {
                if let Err(why) = Self::set_fullscreen(s, &mut win) {
                    game.crash(Error::FullscreenError(why));
                    break;
                }
            }

            if renderer.swapchain_helper.next_image_available() {
                let (width, height) = win.vulkan_drawable_size();
                let extents = RafxExtents2D { width, height };
                if let Err(e) = renderer.draw(extents, 1.0, |canvas, _| {
                    canvas.clear(Self::BACKGROUND);
                    game.draw(canvas);
                }) {
                    game.crash(e.into());
                    break;
                }
                State::with_mut(|x| x.time_state_draw.update());
            }
            State::with_mut(|state| {
                let update_time = state.time_state.last_update().elapsed();
                if target_update_time > update_time {
                    sleep(target_update_time - update_time);
                }
                state.time_state.update();
            });
        }

        Ok(())
    }

    fn set_fullscreen(
        enable: bool,
        win: &mut Sdl2Window,
    ) -> Result<(), String> {
        win.set_fullscreen(if enable {
            FullscreenType::Desktop
        } else {
            FullscreenType::Off
        })?;
        Ok(())
    }

    fn game_handle_event(game: &mut impl Game, event: Sdl2Event) -> bool {
        if let Some(r) = State::with_mut(|x| x.input_state.handle_event(event))
        {
            match r {
                EventHandleResult::Input(event) => game.input(event),
                EventHandleResult::Resized(size) => {
                    game.set_size(Size::new(size.width, size.height))
                }
                EventHandleResult::Exit => {
                    game.close();
                    return true;
                }
            }
        }

        false
    }
}
