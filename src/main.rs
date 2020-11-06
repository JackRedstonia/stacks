use std::ffi::CString;

use skulpin::winit;
use skulpin::winit::dpi::LogicalPosition;
use skulpin::winit::event::MouseButton;
use skulpin::LogicalSize;
use skulpin::{
    app::{TimeState, VirtualKeyCode},
    skia_safe::{
        self, scalar, Canvas, Color4f, Contains, IRect, ISize, Image, ImageInfo, Matrix, Paint,
        Point, RoundOut, Surface, Vector,
    },
    PresentMode,
};

use application::AppBuilder;
use application::AppDrawArgs;
use application::AppHandler;
use application::AppUpdateArgs;
use application::ApplicationError;
use application::InputEvent;
use application::InputState;

mod application;

fn main() {
    AppBuilder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .use_vulkan_debug_layer(false)
        .present_mode_priority(vec![PresentMode::Immediate])
        .inner_size(LogicalSize::new(1280, 720))
        .run(Stacks::new(Transform {
            matrix: Matrix::translate((100.0, 100.0)),
            inner: Rect {
                rect: skia_safe::Rect {
                    top: 0.0,
                    left: 0.0,
                    right: 300.0,
                    bottom: 120.0,
                },
                paint: Paint::new(Color4f::new(0.0, 1.0, 0.0, 1.0), None),
                x: 0,
            },
        }));
}

struct Stacks<T: Component> {
    root: T,
    recycled_matrix_stack: MatrixStack,
}

impl<T: Component> Stacks<T> {
    pub fn new(root: T) -> Self {
        Stacks {
            root,
            recycled_matrix_stack: MatrixStack {
                matrix_stack: Vec::with_capacity(1000),
                matrix: Matrix::translate((0.0, 0.0)),
            },
        }
    }
}

impl<T: Component> AppHandler for Stacks<T> {
    fn update(&mut self, update_args: AppUpdateArgs) {
        for event in &update_args.input_state.events {
            self.root.input(event);
        }
        update_args.input_state.events.clear();

        let mut args = State {
            time_state: update_args.time_state,
            input_state: update_args.input_state,
        };

        self.root.update(&mut args);

        self.recycled_matrix_stack.clear();
    }

    fn draw(&mut self, draw_args: AppDrawArgs) {
        // Generally would want to clear data every time we draw
        draw_args
            .canvas
            .clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        self.root.draw(draw_args.canvas, draw_args.time_state);
        draw_args.canvas.reset_matrix();

        // let typeface = skia_safe::Typeface::from_name("Fira Sans", skia_safe::FontStyle::bold()).unwrap();
        // let font = skia_safe::Font::new(typeface, Some(16.0));
        // draw_args.canvas.draw_str(format!("{}", draw_args.time_state.updates_per_second()), (20.0, 30.0), &font, &Paint::new(Color4f::new(1.0, 1.0, 1.0, 1.0), None));
    }

    fn fatal_error(&mut self, error: &ApplicationError) {
        println!(
            "A fatal error occurred while running application: {}",
            error
        );
    }
}

macro_rules! matrix_stack {
    ($i:expr, $m:expr, $j:block) => {
        $i.save();
        $i.concat($m);
        $j
        $i.restore();
    }
}

pub struct State<'a, 'b> {
    time_state: &'a TimeState,
    input_state: &'b mut InputState,
}

pub struct MatrixStack {
    matrix_stack: Vec<Matrix>,
    matrix: Matrix,
}

impl MatrixStack {
    fn save(&mut self) {
        self.matrix_stack.push(self.matrix);
    }

    fn concat(&mut self, matrix: &Matrix) {
        self.matrix = Matrix::concat(&self.matrix, matrix);
    }

    fn restore(&mut self) {
        if let Some(m) = self.matrix_stack.pop() {
            self.matrix = m;
        }
    }

    fn clear(&mut self) {
        self.matrix_stack.clear();
        self.matrix = Matrix::translate((0.0, 0.0));
    }
}

pub trait Component {
    fn update(&mut self, _state: &mut State) {}
    fn draw(&mut self, _canvas: &mut Canvas, _time_state: &TimeState) {}
    fn input(&mut self, _event: &InputEvent) {}
}

pub struct Rect {
    pub rect: skia_safe::Rect,
    pub paint: Paint,
    pub x: u64,
}

impl Component for Rect {
    fn draw(&mut self, canvas: &mut Canvas, _time_state: &TimeState) {
        canvas.draw_rect(self.rect, &self.paint);
        let typeface =
            skia_safe::Typeface::from_name("Fira Sans", skia_safe::FontStyle::bold()).unwrap();
        let font = skia_safe::Font::new(typeface, Some(16.0));
        canvas.draw_str(
            self.x.to_string(),
            (0.0, self.rect.bottom + 17.0),
            &font,
            &self.paint,
        );
    }

    fn input(&mut self, event: &InputEvent) {
        if let InputEvent::MouseDown(m, LogicalPosition { x, y }) = event {
            if *m == MouseButton::Left && self.rect.contains(Point { x: *x, y: *y }) {
                self.x += 1;
            }
        }
    }
}

pub struct Transform<T: Component> {
    matrix: Matrix,
    inner: T,
}

impl<T: Component> Component for Transform<T> {
    fn draw(&mut self, canvas: &mut Canvas, time_state: &TimeState) {
        matrix_stack!(canvas, &self.matrix, {
            self.inner.draw(canvas, time_state);
        });
    }

    fn input(&mut self, event: &InputEvent) {
        if let Some(p) = event.reverse_map_position(&self.matrix) {
            self.inner.input(&p);
        }
    }
}
