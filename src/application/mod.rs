pub mod input;
pub mod time;

use core::fmt::{Display, Formatter, Result as FmtResult};
use std::error::Error;

use skulpin_renderer::{ash, CoordinateSystem, LogicalSize, PresentMode, RendererBuilder, Size};
use skulpin_renderer_winit::{winit, WinitWindow};

use ash::vk::Result as VkResult;
use winit::{event::Event as WinitEvent, event_loop::{EventLoop, ControlFlow}, window::WindowBuilder};

use input::{EventHandleResult, InputEvent, InputState};
use time::TimeState;

use crate::canvas::Canvas;

#[derive(Debug)]
pub enum ApplicationError {
    RendererError(VkResult),
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            ApplicationError::RendererError(e) => e.fmt(f),
        }
    }
}

impl Error for ApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ApplicationError::RendererError(e) => Some(e),
        }
    }
}

impl From<VkResult> for ApplicationError {
    fn from(result: VkResult) -> Self {
        ApplicationError::RendererError(result)
    }
}

pub trait Application {
    fn update(&mut self, input_state: &InputState, time_state: &TimeState);
    fn draw(&mut self, input_state: &InputState, time_state: &TimeState, canvas: &mut Canvas);
    fn input(&mut self, input_state: &InputState, time_state: &TimeState, event: InputEvent);
    fn close(&mut self);
    fn crash(&mut self, err: ApplicationError);
}

pub struct ApplicationBuilder {
    inner_size: Size,
    window_title: String,
    renderer_builder: RendererBuilder,
}

impl Default for ApplicationBuilder {
    fn default() -> Self {
        ApplicationBuilder::new()
    }
}

impl ApplicationBuilder {
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
    pub fn run<T: 'static + Application + Send>(self, application: T) -> ! {
        ApplicationRunner::run(
            application,
            self.inner_size,
            self.window_title.clone(),
            self.renderer_builder,
        )
    }
}

use std::sync::mpsc::{sync_channel, TryRecvError};
use std::thread::spawn;

pub enum ApplicationEvent<T: 'static> {
    WinitEvent(WinitEvent<'static, T>),
    Crash(ApplicationError),
}

pub enum ApplicationFeedbackEvent {
    Exit,
}

pub struct ApplicationRunner {}

impl ApplicationRunner {
    pub fn run<T: 'static + Application + Send>(
        application: T,
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

        let (canvas_tx, canvas_rx) = sync_channel::<Canvas>(10);
        let (event_tx, event_rx) = sync_channel::<ApplicationEvent<_>>(10);
        let (feedback_tx, feedback_rx) = sync_channel::<ApplicationFeedbackEvent>(10);

        let input_state = InputState::new(&winit_window);
        spawn(move || {
            let mut input_state = input_state;
            let mut application = application;
            let mut time_state = TimeState::new();

            let mut canvas_cap = 200_000;
            let target_update_time = std::time::Duration::MILLISECOND;
            while let Ok(event) = event_rx.recv() {
                match event {
                    ApplicationEvent::WinitEvent(event) => {
                        if let Some(r) = input_state.handle_event(&event) {
                            match r {
                                EventHandleResult::Input(event) => {
                                    application.input(&input_state, &time_state, event)
                                }
                                // TODO: instead of exiting immediately, we (ideally blockingly) call application::close
                                // and wait for it to close everything before exiting ourselves
                                EventHandleResult::Exit => {
                                    application.close();
                                    feedback_tx
                                        .send(ApplicationFeedbackEvent::Exit)
                                        .expect("Failed to send feedback event to draw thread");
                                    break;
                                }
                            }
                        }

                        match event {
                            WinitEvent::MainEventsCleared => {
                                application.update(&input_state, &time_state);
                                let t = time_state.last_update().elapsed();
                                if target_update_time > t {
                                    std::thread::sleep(target_update_time - t);
                                }
                                time_state.update();

                                let mut canvas = Canvas::with_capacity(canvas_cap);
                                application.draw(&input_state, &time_state, &mut canvas);
                                canvas_cap = canvas_cap.max(canvas.capacity());
                                canvas_tx
                                    .send(canvas)
                                    .expect("Failed to send canvas to draw thread");
                            }
                            _ => {}
                        }
                    }
                    ApplicationEvent::Crash(e) => {
                        application.crash(e);
                        feedback_tx
                            .send(ApplicationFeedbackEvent::Exit)
                            .expect("Failed to send feedback event to draw thread");
                        break;
                    }
                }
            }
        });

        // Pass control of this thread to winit until the app terminates. If this app wants to quit,
        // the update loop should send the appropriate event via the channel
        event_loop.run(
            move |event, _window_target, control_flow| match feedback_rx.try_recv() {
                Ok(event) => match event {
                    ApplicationFeedbackEvent::Exit => {
                        *control_flow = ControlFlow::Exit;
                    }
                },
                Err(e) => match e {
                    TryRecvError::Empty => {
                        let window = WinitWindow::new(&winit_window);
                        if let Some(event) = event.to_static() {
                            if event_tx.send(ApplicationEvent::WinitEvent(event)).is_err() {
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                        match canvas_rx.try_recv() {
                            Ok(canvas) => {
                                if let Err(e) = renderer.draw(
                                    &window,
                                    |sk_canvas, _coordinate_system_helper| {
                                        canvas.play(sk_canvas);
                                    },
                                ) {
                                    let _ = event_tx.send(ApplicationEvent::Crash(e.into()));
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
}
