use std::ffi::CString;

use skulpin::{
    app::{TimeState, VirtualKeyCode},
    PresentMode,
    skia_safe::{
        self, Canvas, Contains, Image, ImageInfo, IRect, ISize, Matrix, Paint, Point, RoundOut,
        scalar, Surface, Vector,
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
            matrix: {
                let mut m = Matrix::new_trans((200.0, 200.0));
                m
            },
            rect: skia_safe::Rect {
                left: 0.0,
                top: 0.0,
                right: 250.0,
                bottom: 250.0
            },
            paint: {
                let mut p = Paint::new(skia_safe::Color4f::from(skia_safe::Color::LIGHT_GRAY), None);
                p.set_anti_alias(true);
                p
            }
        }));
}

struct Stacks<T: Node> {
    target_framerate: u64,
    // root_node: Box<dyn Node>,
    root_node: NodeWrapper<T>,
    recycled_matrix_stack: MatrixStack,
}

// Matrix::new_all(
// 2.0,
// 0.7,
// 0.2,
// 0.0,
// 1.3,
// 2.4,
// 0.0, 0.0, 0.0,
// )

impl<T: Node> Stacks<T> {
    pub fn new(root_node: T) -> Self {
        Stacks {
            target_framerate: 120,
            // root_node: Box::new(RedrawManager::new(Container {
            //     matrix: m,
            //     children: vec![Wiggle {
            //         matrix: Matrix::new_trans((0.0, 0.0)),
            //         inner: Container {
            //             matrix: Matrix::new_trans((0.0, 0.0)),
            //             children: vec![
            //                 Rect {
            //                     matrix: Matrix::new_trans((40.0, 60.0)),
            //                     rect: skia_safe::Rect {
            //                         left: 0.0,
            //                         top: 0.0,
            //                         right: 100.0,
            //                         bottom: 200.0,
            //                     },
            //                     x: 0.0,
            //                     paint: {
            //                         let mut p = Paint::new(
            //                             skia_safe::Color4f::new(0.0, 0.7, 0.0, 1.0),
            //                             None,
            //                         );
            //                         p.set_anti_alias(true);
            //                         p
            //                     },
            //                 },
            //                 Rect {
            //                     matrix: Matrix::new_trans((0.0, 0.0)),
            //                     rect: skia_safe::Rect {
            //                         left: 0.0,
            //                         top: 0.0,
            //                         right: 50.0,
            //                         bottom: 30.0,
            //                     },
            //                     x: 0.0,
            //                     paint: {
            //                         let mut p = Paint::new(
            //                             skia_safe::Color4f::new(0.7, 0.9, 0.0, 1.0),
            //                             None,
            //                         );
            //                         p.set_anti_alias(true);
            //                         p
            //                     },
            //                 },
            //             ],
            //         },
            //     }],
            // })),
            root_node: NodeWrapper::new(root_node),
            recycled_matrix_stack: MatrixStack {
                matrix_stack: Vec::with_capacity(1000),
                matrix: Matrix::new_trans((0.0, 0.0)),
            },
        }
    }
}

impl<T: Node> AppHandler for Stacks<T> {
    fn target_update_rate(&self) -> u64 {
        1000
    }

    fn update(&mut self, update_args: AppUpdateArgs) {
        let mut args = State {
            time_state: update_args.time_state,
            input_state: update_args.input_state,
        };

        self.root_node.inner.update(
            &mut args,
            &mut self.recycled_matrix_stack,
        );

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

        self.root_node.draw(
            draw_args.canvas,
            &mut self.recycled_matrix_stack,
        );
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

pub trait Node {
    fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack);
    fn should_redraw(&self) -> bool;
    fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack);
    fn bounds(&self) -> skia_safe::Rect;
}

pub struct NodeWrapper<T: Node> {
    inner: T,
    surface: Option<Surface>,
    bounds_cache: skia_safe::Rect,
    left_top: Point,
}

impl<T: Node> NodeWrapper<T> {
    fn new(inner: T) -> Self {
        Self {
            inner,
            surface: None,
            bounds_cache: Default::default(),
            left_top: (0.0, 0.0).into(),
        }
    }

    fn draw(&mut self, master_canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
        if self.inner.should_redraw() || self.surface.is_none() {
            let bounds_rect = matrix_stack.matrix.map_rect(self.inner.bounds()).0;
            let bounds: IRect = bounds_rect.round_out();
            self.left_top = (bounds.left, bounds.top).into();

            let info = master_canvas.image_info();
            let mut surface = skia_safe::Surface::new_render_target(
                &mut master_canvas.gpu_context().unwrap(),
                skia_safe::Budgeted::Yes,
                &ImageInfo::new(
                    ISize {
                        width: bounds.right - bounds.left,
                        height: bounds.bottom - bounds.top,
                    },
                    info.color_type(),
                    info.alpha_type(),
                    info.color_space(),
                ),
                None,
                skia_safe::gpu::SurfaceOrigin::TopLeft,
                None,
                false,
            ).unwrap();

            self.bounds_cache = bounds_rect;
            let canvas = surface.canvas();
            canvas.clear(skia_safe::Color::TRANSPARENT);
            matrix_stack!(
                matrix_stack,
                &Matrix::new_trans((-self.left_top.x, -self.left_top.y)),
                {
                    canvas.set_matrix(&matrix_stack.matrix);
                    self.inner.draw(canvas, matrix_stack);
                }
            );

            self.surface = Some(surface);
        }
        master_canvas.draw_image(
            self.surface.as_mut().unwrap().image_snapshot(),
            self.left_top,
            None,
        );
        master_canvas.draw_rect(self.bounds_cache, &{
            let mut p = Paint::new(skia_safe::Color4f::new(0.3, 1.0, 0.5, 0.7), None);
            p.set_anti_alias(true);
            p.set_style(skia_safe::PaintStyle::Stroke);
            p.set_stroke_width(2.0);
            p
        });
    }
}

struct Rect {
    matrix: Matrix,
    rect: skia_safe::Rect,
    paint: Paint,
}

impl Node for Rect {
    fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) {
        // gay
    }

    fn should_redraw(&self) -> bool {
        false
    }

    fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
        matrix_stack!(matrix_stack, &self.matrix, {
            canvas.set_matrix(&matrix_stack.matrix);
            canvas.draw_rect(self.rect, &self.paint);
        });
    }

    fn bounds(&self) -> skia_safe::Rect {
        self.matrix.map_rect(self.rect).0
    }
}


// pub trait Node {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack);
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack);
// }
//
// pub struct RedrawManager<T: RedrawManagedNode> {
//     inner: T,
//     should_redraw: bool,
//     virtual_surface: Option<Surface>,
//     left_top: Point,
// }
//
// impl<T: RedrawManagedNode> RedrawManager<T> {
//     fn new(inner: T) -> Self {
//         Self {
//             inner,
//             should_redraw: true,
//             virtual_surface: None,
//             left_top: (0.0, 0.0).into(),
//         }
//     }
// }
//
// impl<T: RedrawManagedNode> Node for RedrawManager<T> {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) {
//         self.should_redraw = self.inner.update(state, matrix_stack);
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         if self.should_redraw || self.virtual_surface.is_none() {
//             let bounds = self.inner.bounds();
//             self.left_top = (bounds.left, bounds.top).into();
//             let info = canvas.image_info();
//
//             let mut s = skia_safe::Surface::new_render_target(
//                 &mut canvas.gpu_context().unwrap(),
//                 skia_safe::Budgeted::Yes,
//                 &ImageInfo::new(
//                     ISize {
//                         width: bounds.right - bounds.left,
//                         height: bounds.bottom - bounds.top,
//                     },
//                     info.color_type(),
//                     info.alpha_type(),
//                     info.color_space(),
//                 ),
//                 None,
//                 skia_safe::gpu::SurfaceOrigin::TopLeft,
//                 None,
//                 false,
//             )
//             .unwrap();
//
//             let c = s.canvas();
//             c.clear(skia_safe::Color::from_argb(0, 0, 0, 0));
//             matrix_stack!(
//                 matrix_stack,
//                 &Matrix::new_trans((-self.left_top.x, -self.left_top.y)),
//                 {
//                     canvas.set_matrix(&matrix_stack.matrix);
//                     self.inner.draw(c, matrix_stack);
//                 }
//             );
//
//             self.virtual_surface = Some(s);
//         }
//         canvas.draw_image(
//             self.virtual_surface.as_mut().unwrap().image_snapshot(),
//             self.left_top * 2.0,
//             None,
//         );
//     }
// }

// pub trait RedrawManagedNode {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) -> bool;
//     fn bounds(&self) -> IRect;
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack);
// }
//
// fn irect_to_rect(irect: IRect) -> skia_safe::Rect {
//     skia_safe::Rect {
//         left: irect.left as scalar,
//         top: irect.top as scalar,
//         right: irect.right as scalar,
//         bottom: irect.bottom as scalar,
//     }
// }
//
// struct Rect {
//     matrix: Matrix,
//     rect: skia_safe::Rect,
//     x: scalar,
//     paint: Paint,
// }
//
// impl Node for Rect {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) {
//         if self.x > 0.0 {
//             self.x -= state.time_state.previous_update_time().as_secs_f32() * 2.0;
//             if self.x < 0.0 {
//                 self.x = 0.0;
//             }
//         }
//
//         matrix_stack!(matrix_stack, &self.matrix, {
//             if let Some(m) = matrix_stack.matrix.invert() {
//                 state.input_state.events.iter(|event| {
//                     if let InputEvent::MouseDown(MouseButton::Left, mouse_position) = event {
//                         if self
//                             .rect
//                             .contains(m.map_point((mouse_position.x, mouse_position.y)))
//                         {
//                             self.x = 1.0;
//                             return true;
//                         }
//                     }
//                     return false;
//                 });
//             }
//         });
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         matrix_stack!(matrix_stack, &self.matrix, {
//             canvas.set_matrix(&matrix_stack.matrix);
//             if self.x > 0.0 {
//                 let mut c = self.paint.color4f();
//                 c.r = (c.r + self.x).min(1.0);
//                 c.g = (c.g + self.x).min(1.0);
//                 c.b = (c.b + self.x).min(1.0);
//                 let mut p = Paint::new(c, None);
//                 p.set_anti_alias(true);
//                 canvas.draw_rect(self.rect, &p);
//             } else {
//                 canvas.draw_rect(self.rect, &self.paint);
//             }
//         });
//     }
// }
//
// impl RedrawManagedNode for Rect {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) -> bool {
//         let mut redraw = false;
//
//         if self.x > 0.0 {
//             self.x -= state.time_state.previous_update_time().as_secs_f32() * 2.0;
//             if self.x < 0.0 {
//                 self.x = 0.0;
//             }
//             redraw = true;
//         }
//
//         matrix_stack!(matrix_stack, &self.matrix, {
//             if let Some(m) = matrix_stack.matrix.invert() {
//                 state.input_state.events.iter(|event| {
//                     if let InputEvent::MouseDown(MouseButton::Left, mouse_position) = event {
//                         if self
//                             .rect
//                             .contains(m.map_point((mouse_position.x, mouse_position.y)))
//                         {
//                             self.x = 1.0;
//                             redraw = true;
//                             return true;
//                         }
//                     }
//                     return false;
//                 });
//             }
//         });
//         redraw
//     }
//
//     fn bounds(&self) -> IRect {
//         self.matrix.map_rect(self.rect).0.round_out()
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         matrix_stack!(matrix_stack, &self.matrix, {
//             canvas.set_matrix(&matrix_stack.matrix);
//             if self.x > 0.0 {
//                 let mut c = self.paint.color4f();
//                 c.r = (c.r + self.x).min(1.0);
//                 c.g = (c.g + self.x).min(1.0);
//                 c.b = (c.b + self.x).min(1.0);
//                 let mut p = Paint::new(c, None);
//                 p.set_anti_alias(true);
//                 canvas.draw_rect(self.rect, &p);
//             } else {
//                 canvas.draw_rect(self.rect, &self.paint);
//             }
//         });
//     }
// }
//
// struct Container<T> {
//     matrix: Matrix,
//     children: Vec<T>,
// }
//
// impl<T: Node> Node for Container<T> {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) {
//         matrix_stack!(matrix_stack, &self.matrix, {
//             for c in &mut self.children {
//                 c.update(state, matrix_stack);
//             }
//         });
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         matrix_stack!(matrix_stack, &self.matrix, {
//             canvas.set_matrix(&matrix_stack.matrix);
//             for c in &mut self.children {
//                 c.draw(canvas, matrix_stack);
//             }
//         });
//     }
// }
//
// impl<T: RedrawManagedNode> Container<T> {
//     fn ibounds(&self) -> IRect {
//         if self.children.is_empty() {
//             IRect {
//                 left: 0,
//                 top: 0,
//                 right: 0,
//                 bottom: 0,
//             }
//         } else {
//             let mut i = self.children.iter();
//             let mut r = i.next().unwrap().bounds();
//             for c in i {
//                 let b = c.bounds();
//                 r = IRect {
//                     left: b.left.min(r.left),
//                     top: b.top.min(r.top),
//                     right: b.right.max(r.right),
//                     bottom: b.bottom.max(r.bottom),
//                 };
//             }
//             r
//         }
//     }
// }
//
// impl<T: RedrawManagedNode> RedrawManagedNode for Container<T> {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) -> bool {
//         let mut b = false;
//         matrix_stack!(matrix_stack, &self.matrix, {
//             for c in &mut self.children {
//                 b = c.update(state, matrix_stack) || b;
//             }
//         });
//         b
//     }
//
//     fn bounds(&self) -> IRect {
//         self.matrix.map_rect(irect_to_rect(self.ibounds())).0.round_out()
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         // DEBUG DRAW
//         let mut p = Paint::new(skia_safe::Color4f::new(1.0, 1.0, 1.0, 0.7), None);
//         p.set_anti_alias(true);
//         p.set_style(skia_safe::PaintStyle::Stroke);
//         p.set_stroke_width(2.0);
//         matrix_stack!(matrix_stack, &self.matrix, {
//             canvas.set_matrix(&matrix_stack.matrix);
//             canvas.draw_rect(irect_to_rect(self.ibounds()), &p);
//             for c in &mut self.children {
//                 c.draw(canvas, matrix_stack);
//             }
//         });
//     }
// }
//
// struct Wiggle<T> {
//     matrix: Matrix,
//     inner: T,
// }
//
// impl<T: Node> Node for Wiggle<T> {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) {
//         self.matrix.set_rotate(
//             (state.time_state.total_time().as_secs_f32() * 10.0).sin() * 10.0,
//             None,
//         );
//         matrix_stack!(matrix_stack, &self.matrix, {
//             self.inner.update(state, matrix_stack);
//         });
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         matrix_stack!(matrix_stack, &self.matrix, {
//             canvas.set_matrix(&matrix_stack.matrix);
//             self.inner.draw(canvas, matrix_stack);
//         });
//     }
// }
//
// impl<T: RedrawManagedNode> RedrawManagedNode for Wiggle<T> {
//     fn update(&mut self, state: &mut State, matrix_stack: &mut MatrixStack) -> bool {
//         self.matrix.set_rotate(
//             (state.time_state.total_time().as_secs_f32() * 10.0).sin() * 10.0,
//             None,
//         );
//         matrix_stack!(matrix_stack, &self.matrix, {
//             self.inner.update(state, matrix_stack);
//         });
//         true
//     }
//
//     fn bounds(&self) -> IRect {
//         self.matrix
//             .map_rect(irect_to_rect(self.inner.bounds()))
//             .0
//             .round_out()
//     }
//
//     fn draw(&mut self, canvas: &mut Canvas, matrix_stack: &mut MatrixStack) {
//         matrix_stack!(matrix_stack, &self.matrix, {
//             canvas.set_matrix(&matrix_stack.matrix);
//             self.inner.draw(canvas, matrix_stack);
//         });
//     }
// }
