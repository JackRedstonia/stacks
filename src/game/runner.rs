use core::fmt::{Display, Formatter, Result as FmtResult};
use std::cell::RefCell;
use std::error::Error as StdError;
use std::sync::mpsc::{sync_channel, SyncSender, TryRecvError};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};

use skulpin_renderer::{ash, RendererBuilder, Size};
use skulpin_renderer_winit::{winit, WinitWindow};

use ash::vk::Result as VkResult;
use winit::{
    dpi::{LogicalPosition, PhysicalSize},
    event::{Event as WinitEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowId},
};

use crate::skia::{scalar, Color, Matrix, Picture, PictureRecorder, Rect, Size as SkSize};

use super::input::{EventHandleResult, InputState};
use super::time::TimeState;
use super::Game;
use super::{default_font_set::DefaultFontSet, FontSet};

enum Event<T: 'static> {
    WinitEvent(WinitEvent<'static, T>),
    ScaleFactorChanged(WindowId, f64, PhysicalSize<u32>),
    Crash(Error),
}

enum FeedbackEvent {
    Exit,
}

#[derive(Debug)]
pub enum Error {
    RendererError(VkResult),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Error::RendererError(e) => e.fmt(f),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::RendererError(e) => Some(e),
        }
    }
}

impl From<VkResult> for Error {
    fn from(result: VkResult) -> Self {
        Error::RendererError(result)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    pub font_set: Box<dyn FontSet>,
    id_keeper: u64,
}

impl State {
    const PANIC_MESSAGE: &'static str = "Attempt to get game state while game is uninitialised";
    thread_local!(pub static STATE: RefCell<Option<State>> = RefCell::new(None));

    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&State) -> R,
    {
        Self::STATE.with(|x| f(x.borrow().as_ref().expect(Self::PANIC_MESSAGE)))
    }

    #[inline]
    pub fn with_mut<F, R>(f: F) -> R
    where
        F: FnOnce(&mut State) -> R,
    {
        Self::STATE.with(|x| f(x.borrow_mut().as_mut().expect(Self::PANIC_MESSAGE)))
    }

    pub fn last_update_time() -> Duration {
        Self::STATE.with(|x| {
            x.borrow()
                .as_ref()
                .expect(Self::PANIC_MESSAGE)
                .time_state
                .last_update_time()
        })
    }

    pub fn elapsed() -> Duration {
        Self::STATE.with(|x| {
            x.borrow()
                .as_ref()
                .expect(Self::PANIC_MESSAGE)
                .time_state
                .elapsed()
        })
    }

    pub fn last_update_time_draw() -> Duration {
        Self::STATE.with(|x| {
            x.borrow()
                .as_ref()
                .expect(Self::PANIC_MESSAGE)
                .time_state_draw
                .last_update_time()
        })
    }

    pub fn mouse_position() -> LogicalPosition<scalar> {
        Self::STATE.with(|x| {
            x.borrow()
                .as_ref()
                .expect(Self::PANIC_MESSAGE)
                .input_state
                .mouse_position
        })
    }
}

pub struct Runner;

impl Runner {
    pub const PIC_QUEUE_LENGTH: usize = 1;
    pub const EVENT_QUEUE_SIZE: usize = 8;
    pub const FEEDBACK_QUEUE_SIZE: usize = 1;

    pub const BACKGROUND: Color = Color::from_argb(255, 10, 10, 10);

    pub fn run<F, T>(
        game: F,
        inner_size: Size,
        window_title: String,
        renderer_builder: RendererBuilder,
    ) -> !
    where
        F: 'static + Send + FnOnce() -> T,
        T: Game,
    {
        let winit_size = match inner_size {
            Size::Physical(physical_size) => winit::dpi::Size::Physical(
                winit::dpi::PhysicalSize::new(physical_size.width, physical_size.height),
            ),
            Size::Logical(logical_size) => winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                logical_size.width as f64,
                logical_size.height as f64,
            )),
        };

        // Create the event loop
        let event_loop = EventLoop::<()>::with_user_event();

        let (pic_tx, pic_rx) = sync_channel(Self::PIC_QUEUE_LENGTH);
        let (event_tx, event_rx) = sync_channel(Self::EVENT_QUEUE_SIZE);
        let (feedback_tx, feedback_rx) = sync_channel(Self::FEEDBACK_QUEUE_SIZE);

        spawn(move || {
            let input_state = InputState::new(winit_size.to_logical(1.0));
            let time_state = TimeState::new();
            let time_state_draw = TimeState::new();
            State::STATE.with(|x| {
                *x.borrow_mut() = Some(State {
                    input_state,
                    time_state,
                    time_state_draw,
                    font_set: Box::new(DefaultFontSet::new()),
                    id_keeper: 0,
                });
            });

            let mut game = game();

            let target_update_time = Duration::MILLISECOND; // 1000 fps
            loop {
                game.update();
                let mut is_redraw = false;
                loop {
                    match event_rx.try_recv() {
                        Ok(event) => {
                            if Self::handle_event(
                                &mut game,
                                event,
                                &pic_tx,
                                &feedback_tx,
                                &mut is_redraw,
                            ) {
                                return;
                            }
                        }
                        Err(e) => match e {
                            TryRecvError::Empty => break,
                            TryRecvError::Disconnected => return,
                        },
                    }
                }
                State::with_mut(|state| {
                    if !is_redraw {
                        let update_time = state.time_state.last_update().elapsed();
                        if target_update_time > update_time {
                            sleep(target_update_time - update_time);
                        }
                    }
                    state.time_state.update();
                });
            }
        });

        // Create a single window
        let winit_window = WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit_size)
            .build(&event_loop)
            .expect("Failed to create winit window");

        let window = WinitWindow::new(&winit_window);
        let mut renderer = renderer_builder
            .build(&window)
            .expect("Failed to create renderer");

        winit_window.set_cursor_visible(false);
        if event_tx
            .send(Event::ScaleFactorChanged(
                winit_window.id(),
                winit_window.scale_factor(),
                winit_window.inner_size(),
            ))
            .is_err()
        {
            panic!("Failed to send events to game thread, probably already crashed");
        };

        let target_frame_time = Duration::MILLISECOND * 8; // 120 fps
        let mut last_frame = Instant::now();
        event_loop.run(
            move |event, _window_target, control_flow| match feedback_rx.try_recv() {
                Ok(event) => match event {
                    FeedbackEvent::Exit => {
                        *control_flow = ControlFlow::Exit;
                    }
                },
                Err(e) => match e {
                    TryRecvError::Empty => {
                        // TODO: this simply does not work well with
                        // presentation modes other than Immediate.
                        // this results in window resizing and moving being
                        // unbearably laggy if not using Immediate.
                        let frame_time = last_frame.elapsed();
                        if frame_time > target_frame_time {
                            winit_window.request_redraw();
                            last_frame = Instant::now() - (frame_time - target_frame_time);
                        }
                        match event {
                            WinitEvent::WindowEvent {
                                window_id,
                                event:
                                    WindowEvent::ScaleFactorChanged {
                                        scale_factor,
                                        new_inner_size,
                                    },
                            } => {
                                if event_tx
                                    .send(Event::ScaleFactorChanged(
                                        window_id,
                                        scale_factor,
                                        *new_inner_size,
                                    ))
                                    .is_err()
                                {
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                            event => {
                                if let Some(event) = event.to_static() {
                                    if event_tx.send(Event::WinitEvent(event)).is_err() {
                                        *control_flow = ControlFlow::Exit;
                                    }
                                }
                            }
                        }
                        match pic_rx.try_recv() {
                            Ok(pic) => {
                                let window = WinitWindow::new(&winit_window);
                                if let Err(e) = renderer.draw(&window, |canvas, _| {
                                    canvas.clear(Self::BACKGROUND);
                                    canvas.draw_picture(pic, Some(&Matrix::default()), None);
                                }) {
                                    let _ = event_tx.send(Event::Crash(e.into()));
                                    *control_flow = ControlFlow::Exit;
                                }
                            }
                            Err(e) => match e {
                                TryRecvError::Empty => {}
                                TryRecvError::Disconnected => *control_flow = ControlFlow::Exit,
                            },
                        }
                    }
                    TryRecvError::Disconnected => *control_flow = ControlFlow::Exit,
                },
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_event<T>(
        game: &mut impl Game,
        event: Event<T>,
        canvas_tx: &SyncSender<Picture>,
        feedback_tx: &SyncSender<FeedbackEvent>,
        is_redraw: &mut bool,
    ) -> bool {
        match event {
            Event::WinitEvent(event) => {
                if let Some(r) = State::with_mut(|x| x.input_state.handle_event(&event)) {
                    match r {
                        EventHandleResult::Input(event) => game.input(event),
                        EventHandleResult::Resized(size) => {
                            game.set_size(SkSize::new(size.width, size.height))
                        }
                        EventHandleResult::Exit => {
                            game.close();
                            feedback_tx
                                .send(FeedbackEvent::Exit)
                                .expect("Failed to send feedback event to draw thread");
                            return true;
                        }
                    }
                }

                if let WinitEvent::RedrawRequested(_) = event {
                    *is_redraw = true;
                    let mut rec = PictureRecorder::new();
                    let bounds = Rect::from_size(State::with(|x| {
                        let w = x.input_state.window_size;
                        (w.width, w.height)
                    }));
                    let canvas = rec.begin_recording(bounds, None);
                    game.draw(canvas);
                    canvas_tx
                        .send(
                            rec.finish_recording_as_picture(None)
                                .expect("Failed to finish recording picture while rendering"),
                        )
                        .expect("Failed to send canvas to draw thread");
                    State::with_mut(|x| x.time_state_draw.update());
                }
            }
            Event::ScaleFactorChanged(window_id, scale_factor, mut new_inner_size) => {
                let e: WinitEvent<()> = WinitEvent::WindowEvent {
                    window_id,
                    event: WindowEvent::ScaleFactorChanged {
                        scale_factor,
                        new_inner_size: &mut new_inner_size,
                    },
                };
                State::with_mut(|x| x.input_state.handle_event(&e));
            }
            Event::Crash(e) => {
                game.crash(e);
                feedback_tx
                    .send(FeedbackEvent::Exit)
                    .expect("Failed to send feedback event to draw thread");
                return true;
            }
        }

        false
    }
}
