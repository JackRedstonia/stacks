use core::fmt::{Display, Formatter, Result as FmtResult};
use std::cell::RefCell;
use std::error::Error as StdError;
use std::sync::mpsc::{sync_channel, SyncSender, TryRecvError};
use std::thread::spawn;
use std::time::Duration;

use skulpin_renderer::{ash, RendererBuilder, Size};
use skulpin_renderer_winit::{winit, WinitWindow};

use ash::vk::Result as VkResult;
use winit::{
    event::Event as WinitEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::skia;

use super::default_font_set::DefaultFontSet;
use super::input::{EventHandleResult, InputState};
use super::time::TimeState;
use super::Game;

use super::canvas::Canvas;

enum Event<T: 'static> {
    WinitEvent(WinitEvent<'static, T>),
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

pub struct State {
    pub input_state: InputState,
    pub time_state: TimeState,
    pub time_state_draw: TimeState,
}

impl State {
    thread_local!(pub static STATE: RefCell<Option<State>> = RefCell::new(None));

    #[inline]
    pub fn with<F, R>(f: F) -> R
    where
        F: FnOnce(&mut State) -> R,
    {
        Self::STATE.with(|x| f(x.borrow_mut().as_mut().unwrap()))
    }

    pub fn last_update_time() -> Duration {
        Self::STATE.with(|x| x.borrow().as_ref().unwrap().time_state.last_update_time())
    }

    pub fn last_update_time_draw() -> Duration {
        Self::STATE.with(|x| {
            x.borrow()
                .as_ref()
                .unwrap()
                .time_state_draw
                .last_update_time()
        })
    }
}

pub struct Runner;

impl Runner {
    pub const CANVAS_QUEUE_LENGTH: usize = 8;
    pub const EVENT_QUEUE_SIZE: usize = 8;
    pub const FEEDBACK_QUEUE_SIZE: usize = 8;

    pub const BACKGROUND: skia::Color = skia::Color::from_argb(255, 10, 10, 10);

    pub fn run<T: 'static + Game + Send>(
        game: T,
        inner_size: Size,
        window_title: String,
        renderer_builder: RendererBuilder,
    ) -> ! {
        // Create the event loop
        let event_loop = EventLoop::<()>::with_user_event();

        let winit_size = match inner_size {
            Size::Physical(physical_size) => winit::dpi::Size::Physical(
                winit::dpi::PhysicalSize::new(physical_size.width, physical_size.height),
            ),
            Size::Logical(logical_size) => winit::dpi::Size::Logical(winit::dpi::LogicalSize::new(
                logical_size.width as f64,
                logical_size.height as f64,
            )),
        };

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

        let (canvas_tx, canvas_rx) = sync_channel(Self::CANVAS_QUEUE_LENGTH);
        let (event_tx, event_rx) = sync_channel(Self::EVENT_QUEUE_SIZE);
        let (feedback_tx, feedback_rx) = sync_channel(Self::FEEDBACK_QUEUE_SIZE);

        let input_state = InputState::new(&winit_window);
        spawn(move || {
            let mut game = game;

            State::STATE.with(|x| {
                let input_state = input_state;
                let time_state = TimeState::new();
                let time_state_draw = TimeState::new();
                *x.borrow_mut() = Some(State {
                    input_state,
                    time_state,
                    time_state_draw,
                });
            });

            let mut canvas_cap = 200_000;
            let target_update_time = std::time::Duration::MILLISECOND; // 1000 fps
            loop {
                let mut is_redraw = false;
                loop {
                    match event_rx.try_recv() {
                        Ok(event) => {
                            if Self::handle_event(
                                &mut game,
                                event,
                                &canvas_tx,
                                &feedback_tx,
                                &mut canvas_cap,
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
                game.update();
                State::with(|state| {
                    if !is_redraw {
                        let update_time = state.time_state.last_update().elapsed();
                        if target_update_time > update_time {
                            std::thread::sleep(target_update_time - update_time);
                        }
                    }
                    state.time_state.update();
                });
            }
        });

        let target_frame_time = std::time::Duration::MILLISECOND * 8; // 120 fps
        let mut last_frame = std::time::Instant::now();

        let font_set = DefaultFontSet::new();

        winit_window.set_cursor_visible(false);

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
                            last_frame =
                                std::time::Instant::now() - (frame_time - target_frame_time);
                        }
                        if let Some(event) = event.to_static() {
                            if event_tx.send(Event::WinitEvent(event)).is_err() {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        match canvas_rx.try_recv() {
                            Ok(canvas) => {
                                let window = WinitWindow::new(&winit_window);
                                if let Err(e) = renderer.draw(
                                    &window,
                                    |sk_canvas, _coordinate_system_helper| {
                                        sk_canvas.clear(Self::BACKGROUND);
                                        canvas.play(sk_canvas, &font_set);
                                    },
                                ) {
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
        canvas_tx: &SyncSender<Canvas>,
        feedback_tx: &SyncSender<FeedbackEvent>,
        canvas_cap: &mut usize,
        is_redraw: &mut bool,
    ) -> bool {
        match event {
            Event::WinitEvent(event) => {
                if let Some(r) = State::with(|x| x.input_state.handle_event(&event)) {
                    match r {
                        EventHandleResult::Input(event) => game.input(event),
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
                    let mut canvas = Canvas::with_capacity(*canvas_cap);
                    game.draw(&mut canvas);
                    *canvas_cap = (*canvas_cap).max(canvas.capacity());
                    canvas_tx
                        .send(canvas)
                        .expect("Failed to send canvas to draw thread");
                    State::with(|x| x.time_state_draw.update());
                }
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
