use skulpin_renderer::{CoordinateSystem, LogicalSize, PresentMode, RendererBuilder, Size};

use super::{runner::Runner, Game};

pub struct Builder {
    inner_size: Size,
    window_title: String,
    renderer_builder: RendererBuilder,
}

impl Default for Builder {
    fn default() -> Self {
        Builder::new()
    }
}

impl Builder {
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

    /// Start the app.
    /// This does not return because winit does not return. For consistency, we use the
    /// crash() callback on the passed in `Game`.
    pub fn run<F, T>(self, game: F) -> !
    where
        F: 'static + Send + FnOnce() -> T,
        T: Game,
    {
        Runner::run(
            game,
            self.inner_size,
            self.window_title.clone(),
            self.renderer_builder
                .present_mode_priority(vec![PresentMode::Immediate]),
        )
    }
}
