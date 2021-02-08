use core::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error as StdError;
use std::sync::mpsc::{sync_channel, SyncSender, TryRecvError, TrySendError};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use std::{cell::RefCell, sync::mpsc::Receiver};

use crate::skia::{Color, Matrix, Picture, PictureRecorder, Point, Rect, Size};

use super::input::{EventHandleResult, InputState};
use super::time::TimeState;
use super::Game;

use sdl2::{event::Event as Sdl2Event, video::FullscreenType};
use skulpin_renderer::{
    ash::vk::Result as VkResult, LogicalSize, RendererBuilder,
};
use skulpin_renderer_sdl2::{sdl2, Sdl2Window};

enum Event {
    CanvasReady,
    RefreshRateChange(i32),
    Sdl2(Sdl2Event),
    Crash(Error),
}

enum FeedbackEvent<T> {
    GameError(T),
    Exit,
    SetFullscreen(bool),
}

#[derive(Debug)]
pub enum Error {
    RendererError(VkResult),
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

impl From<VkResult> for Error {
    fn from(result: VkResult) -> Self {
        Error::RendererError(result)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

impl std::hash::Hash for ID {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.0);
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
}

pub struct Runner;

impl Runner {
    const EVENT_QUEUE_SIZE: usize = 8;
    const FEEDBACK_QUEUE_SIZE: usize = 1;

    const MAIN_THREAD_SLEEP_DURATION: Duration = Duration::from_millis(1);

    const BACKGROUND: Color = Color::from_argb(255, 10, 10, 10);

    pub fn run<F, T, E>(
        game: F,
        size: LogicalSize,
        title: &str,
        renderer_builder: RendererBuilder,
    ) -> Result<(), E>
    where
        F: 'static + Send + FnOnce() -> Result<T, E>,
        T: Game,
        E: Send + 'static,
    {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let sdl_video = sdl.video().expect("Failed to initialize SDL2 video");

        let mut sdl_window = sdl_video
            .window(title, size.width, size.height)
            .resizable()
            .build()
            .expect("Failed to create game window");

        // Buffer size is 1 for pictures, we need pressure to immediately push
        // back at the game thread if we're overloading the swapchain with too
        // many frames.
        let (pic_tx, pic_rx) = sync_channel(1);
        let (event_tx, event_rx) = sync_channel(Self::EVENT_QUEUE_SIZE);
        let (feedback_tx, feedback_rx) =
            sync_channel(Self::FEEDBACK_QUEUE_SIZE);

        spawn(move || {
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

            let mut game = match game() {
                Ok(g) => g,
                Err(e) => {
                    feedback_tx
                        .send(FeedbackEvent::GameError(e))
                        .expect("Failed to send GameError to main thread");
                    return;
                }
            };
            game.set_size(State::STATE.with(|x| {
                x.borrow().as_ref().unwrap().input_state.window_size
            }));
            Self::game_thread(game, event_rx, pic_tx, feedback_tx);
        });

        let mut renderer = renderer_builder
            .build(&Sdl2Window::new(&sdl_window))
            .expect("Failed to create renderer");

        let mut event_pump =
            sdl.event_pump().expect("Failed to create SDL2 event pump");

        event_tx
            .send(Event::CanvasReady)
            .expect("Failed to send canvas ready event to game thread");

        let mut current_refresh_rate = 60;

        'events: loop {
            match feedback_rx.try_recv() {
                Ok(event) => match event {
                    FeedbackEvent::GameError(err) => {
                        return Err(err);
                    }
                    FeedbackEvent::Exit => {
                        break 'events;
                    }
                    FeedbackEvent::SetFullscreen(f) => {
                        if let Err(e) = sdl_window.set_fullscreen(if f {
                            FullscreenType::Desktop
                        } else {
                            FullscreenType::Off
                        }) {
                            let _ = event_tx
                                .send(Event::Crash(Error::FullscreenError(e)));
                            break 'events;
                        }
                    }
                },
                Err(e) => {
                    match e {
                        TryRecvError::Empty => {
                            for event in event_pump.poll_iter() {
                                if event_tx.send(Event::Sdl2(event)).is_err() {
                                    break 'events;
                                }
                            }
                            match pic_rx.try_recv() {
                                Ok(pic) => {
                                    let skulpin_window =
                                        Sdl2Window::new(&sdl_window);
                                    if let Err(e) = renderer.draw(
                                        &skulpin_window,
                                        |canvas, _| {
                                            canvas.clear(Self::BACKGROUND);
                                            canvas.draw_picture(
                                                pic,
                                                Some(&Matrix::default()),
                                                None,
                                            );
                                        },
                                    ) {
                                        let _ = event_tx
                                            .send(Event::Crash(e.into()));
                                        break 'events;
                                    }
                                }
                                Err(e) => match e {
                                    TryRecvError::Empty => {
                                        if let Ok(mode) =
                                            sdl_window.display_mode()
                                        {
                                            let rr = mode.refresh_rate;
                                            if rr != current_refresh_rate {
                                                if event_tx.send(Event::RefreshRateChange(rr)).is_err() {
                                                    break 'events;
                                                }
                                                current_refresh_rate = rr;
                                            }
                                        }
                                        sleep(Self::MAIN_THREAD_SLEEP_DURATION)
                                    }
                                    TryRecvError::Disconnected => break 'events,
                                },
                            }
                        }
                        TryRecvError::Disconnected => break 'events,
                    }
                }
            }
        }

        Ok(())
    }

    fn calc_target_frame_time(framerate: f64) -> Duration {
        Duration::from_secs_f64(1.0 / framerate + 0.0005)
    }

    fn game_thread<E>(
        mut game: impl Game,
        event_rx: Receiver<Event>,
        pic_tx: SyncSender<Picture>,
        feedback_tx: SyncSender<FeedbackEvent<E>>,
    ) {
        let target_update_time = Duration::from_millis(1); // 1000 fps

        let mut canvas_ready = false;
        let mut target_frame_time = Self::calc_target_frame_time(60.0);
        let mut last_frame = Instant::now();

        loop {
            game.update();
            let mut is_redraw = false;
            loop {
                match event_rx.try_recv() {
                    Ok(event) => {
                        if Self::handle_event(
                            &mut game,
                            event,
                            &feedback_tx,
                            &mut canvas_ready,
                            &mut target_frame_time,
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
            let frame_time = last_frame.elapsed();
            if frame_time > target_frame_time {
                last_frame = Instant::now() - (frame_time - target_frame_time);
                // This is a rather cruddy way of detecting if we're sending
                // too many frames, but it works rather well.
                // We simply skip this frame if so. (but continue the clock)
                if frame_time < target_frame_time * 3 / 2 {
                    is_redraw = true;
                    if canvas_ready {
                        let mut rec = PictureRecorder::new();
                        let bounds = Rect::from_size(State::with(|x| {
                            let w = x.input_state.window_size;
                            (w.width, w.height)
                        }));
                        let canvas = rec.begin_recording(bounds, None);
                        game.draw(canvas);
                        if let Err(why) = pic_tx.try_send(
                            rec.finish_recording_as_picture(None)
                                .expect("Failed to finish recording picture while rendering"),
                        ) {
                            match why {
                                // Skip any unsent frames, just in case the renderer
                                // fails to catch up, and to prevent lockups.
                                TrySendError::Full(_) => {}
                                TrySendError::Disconnected(_) => {
                                    panic!(
                                        "Failed to send canvas to draw thread (disconnected channel)"
                                    )
                                }
                            }
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
                        feedback_tx.send(FeedbackEvent::SetFullscreen(s)).expect(
                            "Failed to send set fullscreen event to main thread",
                        );
                    }
                    State::with_mut(|x| x.time_state_draw.update());
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
    }

    fn handle_event<E>(
        game: &mut impl Game,
        event: Event,
        feedback_tx: &SyncSender<FeedbackEvent<E>>,
        canvas_ready: &mut bool,
        target_frame_time: &mut Duration,
    ) -> bool {
        match event {
            Event::CanvasReady => {
                *canvas_ready = true;
            }
            Event::RefreshRateChange(rr) => {
                *target_frame_time = Self::calc_target_frame_time(rr as f64);
            }
            Event::Sdl2(event) => {
                if let Some(r) =
                    State::with_mut(|x| x.input_state.handle_event(&event))
                {
                    match r {
                        EventHandleResult::Input(event) => game.input(event),
                        EventHandleResult::Resized(size) => {
                            game.set_size(Size::new(size.width, size.height))
                        }
                        EventHandleResult::Exit => {
                            game.close();
                            feedback_tx.send(FeedbackEvent::Exit).expect(
                                "Failed to send feedback event to draw thread",
                            );
                            return true;
                        }
                    }
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
