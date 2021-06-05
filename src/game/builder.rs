use super::{runner::Runner, Game};

use glutin::dpi::LogicalSize;

pub struct Builder<'a> {
    inner_size: LogicalSize<f64>,
    window_title: &'a str,
}

impl<'a> Default for Builder<'a> {
    fn default() -> Self {
        Builder::new()
    }
}

impl<'a> Builder<'a> {
    /// Construct the app builder initialized with default options
    pub fn new() -> Self {
        Self {
            inner_size: LogicalSize::new(1280.0, 720.0),
            window_title: "Stacks",
        }
    }

    /// Specifies the inner size of the window. Both physical and logical coordinates are accepted.
    pub fn inner_size(mut self, inner_size: LogicalSize<f64>) -> Self {
        self.inner_size = inner_size;
        self
    }

    /// Specifies the title that the window will be created with
    pub fn window_title(mut self, title: &'a str) -> Self {
        self.window_title = title;
        self
    }

    /// Start the app.
    pub fn run<F, T, E>(self, game: F) -> E
    where
        F: FnOnce() -> Result<T, E>,
        T: Game + 'static,
    {
        Runner::run(game, self.inner_size, self.window_title)
    }
}
