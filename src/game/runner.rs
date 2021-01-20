use core::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error as StdError;
use std::sync::mpsc::{sync_channel, SyncSender, TryRecvError, TrySendError};
use std::thread::{sleep, spawn};
use std::time::{Duration, Instant};
use std::{cell::RefCell, sync::mpsc::Receiver};

use crate::{
    framework::widgets::{Font, FontStyle},
    skia::{Color, Font as SkFont, Matrix, Picture, PictureRecorder, Point, Rect, Size},
};

use super::input::{EventHandleResult, InputState};
use super::time::TimeState;
use super::Game;
use super::{default_font_set::DefaultFontSet, FontSet};

use sdl2::event::Event as Sdl2Event;
use skulpin_renderer::{ash::vk::Result as VkResult, LogicalSize, RendererBuilder};
use skulpin_renderer_sdl2::{sdl2, Sdl2Window};

use soloud::{
    AudioExt, Backend as SoloudBackend, Bus as SoloudBus, Handle as SoloudHandle, Soloud,
    SoloudError, SoloudFlag,
};

enum Event {
    Sdl2Event(Sdl2Event),
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
    soloud: Soloud,
    default_sound_bus: (SoloudBus, SoloudHandle),
}

#[derive(Clone, Copy, PartialEq)]
pub enum AudioBus {
    Default,
}

impl Default for AudioBus {
    fn default() -> Self {
        Self::Default
    }
}

impl AudioBus {
    pub fn get_fft(self) -> Vec<f32> {
        State::with(|x| self.to_bus(x).calc_fft())
    }

    fn to_bus(self, state: &State) -> &SoloudBus {
        match self {
            Self::Default => &state.default_sound_bus.0,
        }
    }
}

impl State {
    const PANIC_MESSAGE: &'static str = "Attempt to get game state while game is uninitialised";
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
        Self::STATE.with(|x| f(x.borrow_mut().as_mut().expect(Self::PANIC_MESSAGE)))
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

    pub fn mouse_position() -> Point {
        Self::with(|x| x.input_state.mouse_position)
    }

    pub fn get_font_set(font: &Font, style: &FontStyle) -> SkFont {
        Self::with(|x| x.font_set.get(font, style))
    }

    pub fn get_default_font_set(style: &FontStyle) -> SkFont {
        Self::with(|x| x.font_set.get_default(style))
    }

    pub fn play_sound<T>(sound: &T, bus: Option<AudioBus>) -> SoloudHandle
    where
        T: AudioExt,
    {
        Self::play_sound_ex(sound, None, None, None, bus.unwrap_or_default())
    }

    pub fn play_sound_ex<T>(
        sound: &T,
        volume: Option<f32>,
        pan: Option<f32>,
        paused: Option<bool>,
        bus: AudioBus,
    ) -> SoloudHandle
    where
        T: AudioExt,
    {
        Self::with(|x| {
            bus.to_bus(x).play_ex(
                sound,
                volume.unwrap_or(1.0),
                pan.unwrap_or(0.0),
                paused.unwrap_or(false),
            )
        })
    }

    pub fn play_sound_clocked<T>(time: f64, sound: &T) -> SoloudHandle
    where
        T: AudioExt,
    {
        Self::with(|x| x.soloud.play_clocked(time, sound))
    }

    pub fn play_sound_clocked_ex<T>(
        time: f64,
        sound: &T,
        volume: Option<f32>,
        pan: Option<f32>,
        bus: AudioBus,
    ) -> SoloudHandle
    where
        T: AudioExt,
    {
        Self::with(|x| {
            bus.to_bus(x)
                .play_clocked_ex(time, sound, volume.unwrap_or(1.0), pan.unwrap_or(0.0))
        })
    }

    pub fn play_sound_handle(handle: SoloudHandle) {
        Self::with_mut(|x| x.soloud.set_pause(handle, false));
    }

    pub fn pause_sound_handle(handle: SoloudHandle) {
        Self::with_mut(|x| x.soloud.set_pause(handle, true));
    }

    pub fn set_playing_sound_handle(handle: SoloudHandle, playing: bool) {
        Self::with_mut(|x| x.soloud.set_pause(handle, !playing))
    }

    pub fn toggle_playing_sound_handle(handle: SoloudHandle) -> bool {
        Self::with_mut(|x| {
            let was_paused = x.soloud.pause(handle);
            x.soloud.set_pause(handle, !was_paused);
            was_paused
        })
    }

    pub fn is_playing_sound_handle(handle: SoloudHandle) -> bool {
        // The method is called "pause" but it actually returns a boolean
        // indicating whether the handle is paused for some reason.
        // Lucky we get to abstract it out here so users of the library
        // doesn't get confused.
        !Self::with(|x| x.soloud.pause(handle))
    }

    pub fn seek_sound_handle(handle: SoloudHandle, seconds: f64) -> Result<(), SoloudError> {
        Self::with(|x| x.soloud.seek(handle, seconds))
    }

    pub fn playback_position_sound_handle(handle: SoloudHandle) -> f64 {
        Self::with(|x| x.soloud.stream_position(handle))
    }

    pub fn get_sound_master_fft() -> Vec<f32> {
        Self::with(|x| x.soloud.calc_fft())
    }
}

pub struct Runner;

impl Runner {
    pub const PIC_QUEUE_LENGTH: usize = 1;
    pub const EVENT_QUEUE_SIZE: usize = 8;
    pub const FEEDBACK_QUEUE_SIZE: usize = 1;

    pub const BACKGROUND: Color = Color::from_argb(255, 10, 10, 10);

    pub fn run<F, T>(game: F, size: LogicalSize, title: &str, renderer_builder: RendererBuilder)
    where
        F: 'static + Send + FnOnce() -> T,
        T: Game,
    {
        let sdl = sdl2::init().expect("Failed to initialize SDL2");
        let sdl_video = sdl.video().expect("Failed to initialize SDL2 video");

        let sdl_window = sdl_video
            .window(title, size.width, size.height)
            .resizable()
            .build()
            .expect("Failed to create game window");

        let skulpin_window = Sdl2Window::new(&sdl_window);

        let (pic_tx, pic_rx) = sync_channel(Self::PIC_QUEUE_LENGTH);
        let (event_tx, event_rx) = sync_channel(Self::EVENT_QUEUE_SIZE);
        let (feedback_tx, feedback_rx) = sync_channel(Self::FEEDBACK_QUEUE_SIZE);

        spawn(move || {
            let soloud = Soloud::new(
                SoloudFlag::ClipRoundoff | SoloudFlag::EnableVisualization,
                SoloudBackend::Auto,
                44_100,
                256,
                2,
            )
            .expect("Failed to initialize SoLoud");
            let default_bus = SoloudBus::default();
            default_bus.set_visualize_enable(true);
            let default_bus_instance = soloud.play(&default_bus);
            let input_state = InputState::new(size);
            let time_state = TimeState::new();
            let time_state_draw = TimeState::new();
            State::STATE.with(|x| {
                *x.borrow_mut() = Some(State {
                    input_state,
                    time_state,
                    time_state_draw,
                    font_set: Box::new(DefaultFontSet::new()),
                    id_keeper: 0,
                    soloud,
                    default_sound_bus: (default_bus, default_bus_instance),
                });
            });

            let mut game = game();
            game.set_size(
                State::STATE.with(|x| x.borrow().as_ref().unwrap().input_state.window_size),
            );
            Self::game_thread(game, event_rx, pic_tx, feedback_tx);
        });

        let mut renderer = renderer_builder
            .build(&skulpin_window)
            .expect("Failed to create renderer");

        let mut event_pump = sdl.event_pump().expect("Failed to create SDL2 event pump");

        'events: loop {
            match feedback_rx.try_recv() {
                Ok(event) => match event {
                    FeedbackEvent::Exit => {
                        break 'events;
                    }
                },
                Err(e) => match e {
                    TryRecvError::Empty => {
                        for event in event_pump.poll_iter() {
                            if event_tx.send(Event::Sdl2Event(event)).is_err() {
                                break 'events;
                            }
                        }
                        match pic_rx.try_recv() {
                            Ok(pic) => {
                                if let Err(e) = renderer.draw(&skulpin_window, |canvas, _| {
                                    canvas.clear(Self::BACKGROUND);
                                    canvas.draw_picture(pic, Some(&Matrix::default()), None);
                                }) {
                                    let _ = event_tx.send(Event::Crash(e.into()));
                                    break 'events;
                                }
                            }
                            Err(e) => match e {
                                TryRecvError::Empty => sleep(Duration::MILLISECOND),
                                TryRecvError::Disconnected => break 'events,
                            },
                        }
                    }
                    TryRecvError::Disconnected => break 'events,
                },
            }
        }
    }

    fn game_thread(
        mut game: impl Game,
        event_rx: Receiver<Event>,
        pic_tx: SyncSender<Picture>,
        feedback_tx: SyncSender<FeedbackEvent>,
    ) {
        let target_update_time = Duration::MILLISECOND; // 1000 fps
        let target_frame_time = Duration::MILLISECOND * 8; // 120 fps
        let mut last_frame = Instant::now();

        loop {
            game.update();
            let mut is_redraw = false;
            // TODO: is this loop the cause of bad VSync?
            loop {
                match event_rx.try_recv() {
                    Ok(event) => {
                        if Self::handle_event(&mut game, event, &feedback_tx) {
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
                is_redraw = true;
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
                            panic!("Failed to send canvas to draw thread (disconnected channel)")
                        }
                    }
                }
                State::with_mut(|x| x.time_state_draw.update());
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

    fn handle_event(
        game: &mut impl Game,
        event: Event,
        feedback_tx: &SyncSender<FeedbackEvent>,
    ) -> bool {
        match event {
            Event::Sdl2Event(event) => {
                if let Some(r) = State::with_mut(|x| x.input_state.handle_event(&event)) {
                    match r {
                        EventHandleResult::Input(event) => game.input(event),
                        EventHandleResult::Resized(size) => {
                            game.set_size(Size::new(size.width, size.height))
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
