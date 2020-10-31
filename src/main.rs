use std::ffi::CString;

use skulpin::{
    app::{TimeState, VirtualKeyCode},
    PresentMode,
    skia_safe::{
        self, Canvas, Contains, Image, ImageInfo, IRect, ISize, Matrix, Paint, Point, RoundOut,
        scalar, Surface, Vector, Color4f,
    },
};
use skulpin::LogicalSize;
use skulpin::winit;
use skulpin::winit::event::MouseButton;

use application::AppBuilder;
use application::AppDrawArgs;
use application::AppHandler;
use application::ApplicationError;
use application::AppUpdateArgs;
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
        .run(Stacks::new(Rect {
            width: 100.0,
            height: 120.0,
            paint: Paint::new(Color4f::new(0.0, 1.0, 0.0, 1.0), None),
        }));
}

struct Stacks<T: Drawable> {
    target_framerate: u64,
    root: T,
    recycled_matrix_stack: MatrixStack,
}

impl<T: Drawable> Stacks<T> {
    pub fn new(root: T) -> Self {
        Stacks {
            target_framerate: 120,
            root,
            recycled_matrix_stack: MatrixStack {
                matrix_stack: Vec::with_capacity(1000),
                matrix: Matrix::new_trans((0.0, 0.0)),
            },
        }
    }
}

impl<T: Drawable> AppHandler for Stacks<T> {
    fn target_update_rate(&self) -> u64 {
        1000
    }

    fn update(&mut self, update_args: AppUpdateArgs) {
        let mut args = State {
            time_state: update_args.time_state,
            input_state: update_args.input_state,
        };

        self.recycled_matrix_stack.clear();
    }

    fn target_framerate(&self) -> u64 {
        self.target_framerate
    }

    fn draw(&mut self, draw_args: AppDrawArgs) {
        // Generally would want to clear data every time we draw
        draw_args
            .canvas
            .clear(skia_safe::Color::from_argb(0, 0, 0, 255));
        
        let mut virt = VirtCanvas::new();
        self.root.draw(&mut virt);
        virt.play(draw_args.canvas, &mut self.recycled_matrix_stack);
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
        self.matrix = Matrix::new_trans((0.0, 0.0));
    }
}

pub enum DrawCallType {
    Matrix(Matrix, Vec<DrawCallType>),
    Rect(skia_safe::Rect, Paint),
}

pub struct VirtCanvas {
    pub draw_calls: Vec<DrawCallType>,
}

impl VirtCanvas {
    pub fn new() -> Self {
        Self {
            draw_calls: vec![],
        }
    }

    pub fn play(&self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
        Self::play_calls(&self.draw_calls, canvas, matrix_stack);
    }

    pub fn play_calls(calls: &Vec<DrawCallType>, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
        for call in calls {
            use DrawCallType::*;
            match call {
                Matrix(matrix, calls) => {
                    matrix_stack!(matrix_stack, matrix, {
                        Self::play_calls(calls, canvas, matrix_stack);
                    });
                }
                Rect(rect, paint) => {
                    canvas.draw_rect(rect, paint);
                }
            }
        }
    }
}

pub trait Drawable {
    fn draw(&mut self, canvas: &mut VirtCanvas);
}

pub struct Rect {
    pub width: scalar,
    pub height: scalar,
    pub paint: Paint,
}

impl Drawable for Rect {
    fn draw(&mut self, canvas: &mut VirtCanvas) {
        canvas.draw_calls.push(DrawCallType::Rect(skia_safe::Rect {
            top: 0.0,
            left: 0.0,
            right: self.width,
            bottom: self.height,
        }, self.paint.clone()));
    }
}
