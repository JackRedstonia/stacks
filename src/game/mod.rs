pub mod input;
pub mod time;

use core::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error;

use skulpin_renderer::{
    ash, skia_safe, CoordinateSystem, LogicalSize, PresentMode, RendererBuilder, Size,
};
use skulpin_renderer_winit::{winit, WinitWindow};

use ash::vk::Result as VkResult;
use winit::{
    event::Event as WinitEvent,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use input::{EventHandleResult, InputEvent, InputState};
use time::TimeState;

use crate::canvas::{Canvas, FontSet};

#[derive(Debug)]
pub enum GameError {
    RendererError(VkResult),
}

impl Display for GameError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            GameError::RendererError(e) => e.fmt(f),
        }
    }
}

impl Error for GameError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GameError::RendererError(e) => Some(e),
        }
    }
}

impl From<VkResult> for GameError {
    fn from(result: VkResult) -> Self {
        GameError::RendererError(result)
    }
}

pub trait Game {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState);
    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas);
    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: InputEvent);
    fn close(&mut self);
    fn crash(&mut self, err: GameError);
}

pub struct GameBuilder {
    inner_size: Size,
    window_title: String,
    renderer_builder: RendererBuilder,
}

impl Default for GameBuilder {
    fn default() -> Self {
        GameBuilder::new()
    }
}

impl GameBuilder {
    /// Construct the app builder initialized with default options
    pub fn new() -> Self {
        Self {
            inner_size: LogicalSize::new(1280, 720).into(),
            window_title: "Stacks".to_string(),
            renderer_builder: RendererBuilder::new().use_vulkan_debug_layer(false),
        }
    }

    /// Specifies the inner size of the window. Both physical and logical coordinates are accepted.
    pub fn inner_size<S: Into<Size>>(mut self, inner_size: S) -> Self {
        self.inner_size = inner_size.into();
        self
    }

    /// Specifies the title that the window will be created with
    pub fn window_title<T: Into<String>>(mut self, window_title: T) -> Self {
        self.window_title = window_title.into();
        self
    }

    /// Name of the app. This is passed into the vulkan layer. I believe it can hint things to the
    /// vulkan driver, but it's unlikely this makes a real difference. Still a good idea to set this
    /// to something meaningful though.
    pub fn app_name(mut self, app_name: std::ffi::CString) -> Self {
        self.renderer_builder = self.renderer_builder.app_name(app_name);
        self
    }

    /// Determine the coordinate system to use for the canvas. This can be overridden by using the
    /// canvas sizer passed into the draw callback
    pub fn coordinate_system(mut self, coordinate_system: CoordinateSystem) -> Self {
        self.renderer_builder = self.renderer_builder.coordinate_system(coordinate_system);
        self
    }

    /// Specify which PresentMode is preferred. Some of this is hardware/platform dependent and
    /// it's a good idea to read the Vulkan spec. You
    ///
    /// `present_mode_priority` should be a list of desired present modes, in descending order of
    /// preference. In other words, passing `[Mailbox, Fifo]` will direct Skulpin to use mailbox
    /// where available, but otherwise use `Fifo`.
    ///
    /// Since `Fifo` is always available, this is the mode that will be chosen if no desired mode is
    /// available.
    pub fn present_mode_priority(mut self, present_mode_priority: Vec<PresentMode>) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .present_mode_priority(present_mode_priority);
        self
    }

    /// Start the app. `app_handler` must be an implementation of [skulpin::app::AppHandler].
    /// This does not return because winit does not return. For consistency, we use the
    /// fatal_error() callback on the passed in AppHandler.
    pub fn run<T: 'static + Game + Send>(self, game: T) -> ! {
        GameRunner::run(
            game,
            self.inner_size,
            self.window_title.clone(),
            self.renderer_builder,
        )
    }
}

use std::sync::mpsc::{sync_channel, Receiver, SyncSender, TryRecvError};
use std::thread::spawn;

pub enum GameEvent<T: 'static> {
    WinitEvent(WinitEvent<'static, T>),
    Crash(GameError),
}

pub enum GameFeedbackEvent {
    Exit,
}

pub struct GameRunner {}

impl GameRunner {
    pub const CANVAS_QUEUE_LENGTH: usize = 8;
    pub const EVENT_QUEUE_SIZE: usize = 8;
    pub const FEEDBACK_QUEUE_SIZE: usize = 8;

    pub const BACKGROUND: skia_safe::Color = skia_safe::Color::from_argb(255, 10, 10, 10);
}

impl GameRunner {
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
            let mut input_state = input_state;
            let mut game = game;
            let mut time_state = TimeState::new();
            let mut time_state_draw = TimeState::new();

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
                                &mut input_state,
                                &mut time_state,
                                &mut time_state_draw,
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
                game.update(&input_state, &time_state);
                if !is_redraw {
                    let update_time = time_state.last_update().elapsed();
                    if target_update_time > update_time {
                        std::thread::sleep(target_update_time - update_time);
                    }
                }
                time_state.update();
            }
        });

        let target_frame_time = std::time::Duration::MILLISECOND * 4; // 240 fps
        let mut last_frame = std::time::Instant::now();

        let font_set = VanillaFontSet::new();

        event_loop.run(
            move |event, _window_target, control_flow| match feedback_rx.try_recv() {
                Ok(event) => match event {
                    GameFeedbackEvent::Exit => {
                        *control_flow = ControlFlow::Exit;
                    }
                },
                Err(e) => match e {
                    TryRecvError::Empty => {
                        let frame_time = last_frame.elapsed();
                        if frame_time > target_frame_time {
                            winit_window.request_redraw();
                            last_frame =
                                std::time::Instant::now() - (frame_time - target_frame_time);
                        }
                        if let Some(event) = event.to_static() {
                            if event_tx.send(GameEvent::WinitEvent(event)).is_err() {
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
                                    let _ = event_tx.send(GameEvent::Crash(e.into()));
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

    fn handle_event<T>(
        game: &mut impl Game,
        event: GameEvent<T>,
        input_state: &mut InputState,
        time_state: &mut TimeState,
        time_state_draw: &mut TimeState,
        canvas_tx: &SyncSender<Canvas>,
        feedback_tx: &SyncSender<GameFeedbackEvent>,
        canvas_cap: &mut usize,
        is_redraw: &mut bool,
    ) -> bool {
        match event {
            GameEvent::WinitEvent(event) => {
                if let Some(r) = input_state.handle_event(&event) {
                    match r {
                        EventHandleResult::Input(event) => {
                            game.input(&input_state, &time_state, event)
                        }
                        EventHandleResult::Exit => {
                            game.close();
                            feedback_tx
                                .send(GameFeedbackEvent::Exit)
                                .expect("Failed to send feedback event to draw thread");
                            return true;
                        }
                    }
                }

                match event {
                    WinitEvent::RedrawRequested(_) => {
                        *is_redraw = true;
                        let mut canvas = Canvas::with_capacity(*canvas_cap);
                        game.draw(&input_state, &time_state_draw, &mut canvas);
                        *canvas_cap = (*canvas_cap).max(canvas.capacity());
                        canvas_tx
                            .send(canvas)
                            .expect("Failed to send canvas to draw thread");
                        time_state_draw.update();
                    }
                    _ => {}
                }
            }
            GameEvent::Crash(e) => {
                game.crash(e);
                feedback_tx
                    .send(GameFeedbackEvent::Exit)
                    .expect("Failed to send feedback event to draw thread");
                return true;
            }
        }

        false
    }
}

use crate::components::FontStyle;
use skia_safe::{Font, FontStyle as SkFontStyle, Typeface};

// TODO: require argument for font set instead
struct VanillaFontSet {
    default_regular: Font,
    default_bold: Font,
    default_italic: Font,
    default_bold_italic: Font,
}

impl VanillaFontSet {
    fn new() -> Self {
        let family_name = "IBM Plex Sans";
        let size = 16.0;
        Self {
            default_regular: Font::new(
                Typeface::from_name(family_name, SkFontStyle::normal()).unwrap(),
                size,
            ),
            default_bold: Font::new(
                Typeface::from_name(family_name, SkFontStyle::bold()).unwrap(),
                size,
            ),
            default_italic: Font::new(
                Typeface::from_name(family_name, SkFontStyle::italic()).unwrap(),
                size,
            ),
            default_bold_italic: Font::new(
                Typeface::from_name(family_name, SkFontStyle::bold_italic()).unwrap(),
                size,
            ),
        }
    }
}

impl FontSet for VanillaFontSet {
    fn get_default(&self, style: FontStyle) -> &Font {
        match style {
            FontStyle::Regular => &self.default_regular,
            FontStyle::Bold => &self.default_bold,
            FontStyle::Italic => &self.default_italic,
            FontStyle::BoldItalic => &self.default_bold_italic,
        }
    }
}
