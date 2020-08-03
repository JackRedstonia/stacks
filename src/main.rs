mod application;

use application::AppBuilder;
use application::AppDrawArgs;
use application::AppHandler;
use application::AppUpdateArgs;
use application::ApplicationError;
use application::InputState;
use skulpin::LogicalSize;
use skulpin::app::LogicalPosition;
use skulpin::{
    skia_safe::{self, Contains,  scalar, Canvas, Matrix, Paint, Point},
    PresentMode,
    app::{VirtualKeyCode, TimeState},
};

use std::ffi::CString;

use skulpin::winit;

fn main() {
    let example_app = Stacks::new();

    AppBuilder::new()
        .app_name(CString::new("Stacks").unwrap())
        .window_title("Stacks")
        .use_vulkan_debug_layer(false)
        .present_mode_priority(vec![PresentMode::Immediate])
        .inner_size(LogicalSize::new(1280, 720))
        .run(example_app);
}

struct Stacks {
    target_framerate: f64,
    root_node: Box<dyn Node>,
    recycled_matrix_stack: Vec<Matrix>,
}

impl Stacks {
    pub fn new() -> Self {
        Stacks {
            target_framerate: 120.0,
            root_node: Box::new(Wiggle {
                matrix: Matrix::new_trans((0.0, 0.0)),
                inner: Container {
                    matrix: Matrix::new_trans((0.0, 0.0)),
                    children: vec![
                        Rect {
                            matrix: Matrix::new_trans((40.0, 60.0)),
                            rect: skia_safe::Rect {
                                left: 0.0,
                                top: 0.0,
                                right: 100.0,
                                bottom: 200.0,
                            },
                            i: false,
                            paint: {
                                let mut p =
                                    Paint::new(skia_safe::Color4f::new(0.0, 1.0, 0.0, 1.0), None);
                                p.set_anti_alias(true);
                                p
                            },
                        },
                        Rect {
                            matrix: Matrix::new_trans((0.0, 0.0)),
                            rect: skia_safe::Rect {
                                left: 0.0,
                                top: 0.0,
                                right: 50.0,
                                bottom: 30.0,
                            },
                            i: false,
                            paint: {
                                let mut p =
                                    Paint::new(skia_safe::Color4f::new(1.0, 1.0, 0.0, 1.0), None);
                                p.set_anti_alias(true);
                                p
                            },
                        },
                    ],
                },
            }),
            recycled_matrix_stack: Vec::with_capacity(1000),
        }
    }
}

impl AppHandler for Stacks {
    fn target_update_rate(&self) -> f64 {
        1000.0
    }

    fn update(&mut self, update_args: AppUpdateArgs) {
        let input_state = update_args.input_state;

        if input_state.is_key_down(VirtualKeyCode::Escape) {
            update_args.app_control.enqueue_terminate_process();
        }

        let mut args = UpdateArgs {
            time_state: update_args.time_state,
            matrix_stack: &mut self.recycled_matrix_stack,
            matrix: Matrix::new_trans((0.0, 0.0)),
            input_state: update_args.input_state,
            input_taken: false
        };

        self.root_node.update(&mut args);

        self.recycled_matrix_stack.clear();
    }

    fn target_framerate(&self) -> f64 {
        self.target_framerate
    }

    fn draw(&mut self, draw_args: AppDrawArgs) {
        // Generally would want to clear data every time we draw
        draw_args
            .canvas
            .clear(skia_safe::Color::from_argb(0, 0, 0, 255));

        self.root_node.draw(draw_args.canvas);
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

pub struct UpdateArgs<'a, 'b, 'c> {
    time_state: &'a TimeState,
    matrix_stack: &'b mut Vec<Matrix>,
    matrix: Matrix,
    input_state: &'c InputState,
    input_taken: bool,
}

impl<'a, 'b, 'c> UpdateArgs<'a, 'b, 'c> {
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
}

pub trait Node {
    fn update(&mut self, update_args: &mut UpdateArgs);
    fn draw(&mut self, canvas: &mut Canvas);
}

struct Rect {
    matrix: Matrix,
    rect: skia_safe::Rect,
    i: bool,
    paint: Paint,
}

impl Node for Rect {
    fn update(&mut self, update_args: &mut UpdateArgs) {
        let mouse_position: LogicalPosition<scalar> = update_args.input_state.mouse_position()
            .to_logical(update_args.input_state.scale_factor());
        let point = Point::new(mouse_position.x, mouse_position.y);
        matrix_stack!(update_args, &self.matrix, {
            if let Some(m) = update_args.matrix.invert() {
                self.i = self.rect.contains(m.map_point(Point::new(mouse_position.x, mouse_position.y)));
            }
        });
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        matrix_stack!(canvas, &self.matrix, {
            if self.i {
                let mut p = Paint::new(skia_safe::Color4f::new(1.0, 1.0, 1.0, 1.0), None);
                p.set_anti_alias(true);
                canvas.draw_rect(self.rect, &p);
            } else {
                canvas.draw_rect(self.rect, &self.paint);
            }
        });
    }
}

struct Container<T> {
    matrix: Matrix,
    children: Vec<T>,
}

impl<T: Node> Node for Container<T> {
    fn update(&mut self, update_args: &mut UpdateArgs) {
        matrix_stack!(update_args, &self.matrix, {
            for c in &mut self.children {
                c.update(update_args);
            }
        });
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        matrix_stack!(canvas, &self.matrix, {
            for c in &mut self.children {
                c.draw(canvas);
            }
        });
    }
}

struct Wiggle<T> {
    matrix: Matrix,
    inner: T,
}

impl<T: Node> Node for Wiggle<T> {
    fn update(&mut self, update_args: &mut UpdateArgs) {
        self.matrix.set_rotate(
            (update_args.time_state.total_time().as_secs_f32() * 10.0).sin() * 10.0,
            None,
        );
        matrix_stack!(update_args, &self.matrix, {
            self.inner.update(update_args);
        });
    }

    fn draw(&mut self, canvas: &mut Canvas) {
        matrix_stack!(canvas, &self.matrix, {
            self.inner.draw(canvas);
        });
    }
}
