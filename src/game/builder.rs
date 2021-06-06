use super::{runner::run, Game};

use glutin::dpi::LogicalSize;

pub struct Builder<'a> {
    window_size: LogicalSize<f64>,
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
            window_size: LogicalSize::new(1280.0, 720.0),
            window_title: "Stacks",
        }
    }

    /// Specifies the inner size of the window.
    pub fn window_size(mut self, inner_size: LogicalSize<f64>) -> Self {
        self.window_size = inner_size;
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
        run(game, self.window_size, self.window_title)
    }
}
