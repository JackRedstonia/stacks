mod input_state;

use skulpin::ash;
use skulpin::skia_safe;
use skulpin::winit;

pub use input_state::InputEvent;
pub use input_state::InputState;
use skulpin::app::AppControl;
use skulpin::app::TimeState;
use std::ffi::CString;
use std::thread::sleep;

use skulpin::CoordinateSystem;
use skulpin::CoordinateSystemHelper;
use skulpin::LogicalSize;
use skulpin::PhysicalDeviceType;
use skulpin::PresentMode;
use skulpin::RendererBuilder;
use skulpin::Size;
use skulpin::WinitWindow;
use std::time::Duration;

/// Represents an error from creating the renderer
#[derive(Debug)]
pub enum ApplicationError {
    RendererError(ash::vk::Result),
}

impl std::error::Error for ApplicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ApplicationError::RendererError(ref e) => Some(e),
        }
    }
}

impl core::fmt::Display for ApplicationError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        match *self {
            ApplicationError::RendererError(ref e) => e.fmt(fmt),
        }
    }
}

impl From<ash::vk::Result> for ApplicationError {
    fn from(result: ash::vk::Result) -> Self {
        ApplicationError::RendererError(result)
    }
}

pub struct AppUpdateArgs<'a, 'b, 'c> {
    pub app_control: &'a mut AppControl,
    pub input_state: &'b mut InputState,
    pub time_state: &'c TimeState,
}

pub struct AppDrawArgs<'a, 'b, 'c, 'd> {
    pub app_control: &'a AppControl,
    pub input_state: &'b InputState,
    pub time_state: &'c TimeState,
    pub canvas: &'d mut skia_safe::Canvas,
    pub coordinate_system_helper: CoordinateSystemHelper,
}

pub trait AppHandler {
    fn target_update_rate(&self) -> u64;
    fn update(&mut self, pdate_args: AppUpdateArgs);

    fn target_framerate(&self) -> u64;
    fn draw(&mut self, draw_args: AppDrawArgs);

    fn fatal_error(&mut self, error: &ApplicationError);
}

/// Used to configure the app behavior and create the app
pub struct AppBuilder {
    inner_size: Size,
    window_title: String,
    renderer_builder: RendererBuilder,
}

impl Default for AppBuilder {
    fn default() -> Self {
        AppBuilder::new()
    }
}

impl AppBuilder {
    /// Construct the app builder initialized with default options
    pub fn new() -> Self {
        AppBuilder {
            inner_size: LogicalSize::new(1280, 720).into(),
            window_title: "Stacks".to_string(),
            renderer_builder: RendererBuilder::new(),
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
    pub fn app_name(mut self, app_name: CString) -> Self {
        self.renderer_builder = self.renderer_builder.app_name(app_name);
        self
    }

    /// If true, initialize the vulkan debug layers. This will require the vulkan SDK to be
    /// installed and the app will fail to launch if it isn't. This turns on ALL logging. For
    /// more control, see `validation_layer_debug_report_flags()`
    pub fn use_vulkan_debug_layer(mut self, use_vulkan_debug_layer: bool) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .use_vulkan_debug_layer(use_vulkan_debug_layer);
        self
    }

    /// Sets the desired debug layer flags. If any flag is set, the vulkan debug layers will be
    /// loaded, which requires the Vulkan SDK to be installed. The app will fail to launch if it
    /// isn't.
    pub fn validation_layer_debug_report_flags(
        mut self,
        validation_layer_debug_report_flags: ash::vk::DebugReportFlagsEXT,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .validation_layer_debug_report_flags(validation_layer_debug_report_flags);
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

    /// Specify which type of physical device is preferred. It's recommended to read the Vulkan spec
    /// to understand precisely what these types mean
    ///
    /// `physical_device_type_priority` should be a list of desired present modes, in descending
    /// order of preference. In other words, passing `[Discrete, Integrated]` will direct Skulpin to
    /// use the discrete GPU where available, otherwise integrated.
    ///
    /// If the desired device type can't be found, Skulpin will try to use whatever device is
    /// available. By default `Discrete` is favored, then `Integrated`, then anything that's
    /// available. It could make sense to favor `Integrated` over `Discrete` when minimizing
    /// power consumption is important. (Although I haven't tested this myself)
    pub fn physical_device_type_priority(
        mut self,
        physical_device_type_priority: Vec<PhysicalDeviceType>,
    ) -> Self {
        self.renderer_builder = self
            .renderer_builder
            .physical_device_type_priority(physical_device_type_priority);
        self
    }

    /// Easy shortcut to set device type priority to `Integrated`, then `Discrete`, then any.
    pub fn prefer_integrated_gpu(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_integrated_gpu();
        self
    }

    /// Easy shortcut to set device type priority to `Discrete`, then `Integrated`, than any.
    /// (This is the default behavior)
    pub fn prefer_discrete_gpu(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_discrete_gpu();
        self
    }

    /// Prefer using `Fifo` presentation mode. This presentation mode is always available on a
    /// device that complies with the vulkan spec.
    pub fn prefer_fifo_present_mode(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_fifo_present_mode();
        self
    }

    /// Prefer using `Mailbox` presentation mode, and fall back to `Fifo` when not available.
    pub fn prefer_mailbox_present_mode(mut self) -> Self {
        self.renderer_builder = self.renderer_builder.prefer_mailbox_present_mode();
        self
    }

    /// Start the app. `app_handler` must be an implementation of [skulpin::app::AppHandler].
    /// This does not return because winit does not return. For consistency, we use the
    /// fatal_error() callback on the passed in AppHandler.
    pub fn run<T: 'static + AppHandler>(self, app_handler: T) -> ! {
        App::run(
            app_handler,
            self.inner_size,
            self.window_title.clone(),
            self.renderer_builder,
        )
    }
}

/// Constructed by `AppBuilder` which immediately calls `run`.
pub struct App {}

impl App {
    /// Runs the app. This is called by `AppBuilder::run`. This does not return because winit does
    /// not return. For consistency, we use the fatal_error() callback on the passed in AppHandler.
    pub fn run<T: 'static + AppHandler>(
        mut app_handler: T,
        inner_size: Size,
        window_title: String,
        renderer_builder: RendererBuilder,
    ) -> ! {
        // Create the event loop
        let event_loop = winit::event_loop::EventLoop::<()>::with_user_event();

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
        let window_result = winit::window::WindowBuilder::new()
            .with_title(window_title)
            .with_inner_size(winit_size)
            .build(&event_loop);

        let winit_window = match window_result {
            Ok(window) => window,
            Err(e) => panic!("Failed to create winit window: {}", e),
        };

        let window = WinitWindow::new(&winit_window);

        let mut app_control = AppControl::default();
        let mut time_state = TimeState::new();
        let mut input_state = InputState::new(&winit_window);

        let renderer_result = renderer_builder.build(&window);
        let mut renderer = match renderer_result {
            Ok(renderer) => renderer,
            Err(e) => panic!("Failed to create renderer: {}", e),
        };

        // Pass control of this thread to winit until the app terminates. If this app wants to quit,
        // the update loop should send the appropriate event via the channel
        let mut time_accm: u128 = 0;
        event_loop.run(move |event, window_target, control_flow| {
            let window = WinitWindow::new(&winit_window);
            input_state.handle_winit_event(&mut app_control, &event, window_target);

            match event {
                winit::event::Event::MainEventsCleared => {
                    time_state.update();

                    app_handler.update(AppUpdateArgs {
                        app_control: &mut app_control,
                        input_state: &mut input_state,
                        time_state: &time_state,
                    });

                    // Call this to mark the start of the next frame (i.e. "key just down" will return false)
                    input_state.end_frame();

                    let t_update = time_state.previous_update_time();
                    time_accm += t_update.as_nanos();
                    let t_draw = 1_000_000_000 / app_handler.target_framerate() as u128;
                    if time_accm > t_draw {
                        time_accm -= t_draw;
                        winit_window.request_redraw();
                    }

                    let update_time =
                        Duration::from_nanos(1_000_000_000 / app_handler.target_update_rate());
                    if update_time > t_update {
                        *control_flow = winit::event_loop::ControlFlow::WaitUntil(
                            std::time::Instant::now() + update_time - t_update,
                        );
                    }
                }
                winit::event::Event::RedrawRequested(_window_id) => {
                    if let Err(e) = renderer.draw(&window, |canvas, coordinate_system_helper| {
                        app_handler.draw(AppDrawArgs {
                            app_control: &app_control,
                            input_state: &input_state,
                            time_state: &time_state,
                            canvas,
                            coordinate_system_helper,
                        });
                    }) {
                        app_handler.fatal_error(&e.into());
                        app_control.enqueue_terminate_process();
                    }
                }
                _ => {}
            }

            if app_control.should_terminate_process() {
                *control_flow = winit::event_loop::ControlFlow::Exit
            }
        });
    }
}
