use crate::components::{Font, FontStyle};
use skia_safe::{scalar, Canvas as SkCanvas, Matrix, Paint, Point, Rect};
use skulpin_renderer::skia_safe;

pub struct Canvas {
    commands: Vec<Command>,
}

impl Canvas {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            commands: Vec::with_capacity(cap),
        }
    }

    pub fn capacity(&self) -> usize {
        self.commands.capacity()
    }

    pub fn play(&self, canvas: &mut SkCanvas, font_set: &impl FontSet) {
        for command in &self.commands {
            command.execute(canvas, font_set);
        }
    }

    pub fn concat(&mut self, matrix: Matrix) {
        self.commands.push(Command::ConcatMatrix(matrix));
    }

    pub fn save(&mut self) {
        self.commands.push(Command::Save);
    }

    pub fn restore(&mut self) {
        self.commands.push(Command::Restore);
    }

    pub fn draw_rect(&mut self, rect: Rect, paint: &Paint) {
        self.commands.push(Command::Rect(rect, paint.clone()));
    }

    pub fn draw_arc(
        &mut self,
        oval: Rect,
        start: scalar,
        sweep: scalar,
        center: bool,
        paint: &Paint,
    ) {
        self.commands
            .push(Command::Arc(oval, start, sweep, center, paint.clone()));
    }

    pub fn draw_str(
        &mut self,
        text: String,
        origin: impl Into<Point>,
        font: Font,
        style: FontStyle,
        paint: &Paint,
    ) {
        self.commands.push(Command::Str(
            text,
            origin.into(),
            font,
            style,
            paint.clone(),
        ));
    }
}

enum Command {
    ConcatMatrix(Matrix),
    Save,
    Restore,
    Rect(Rect, Paint),
    Arc(Rect, scalar, scalar, bool, Paint),
    Str(String, Point, Font, FontStyle, Paint),
}

impl Command {
    pub fn execute(&self, canvas: &mut SkCanvas, font_set: &impl FontSet) {
        match self {
            Command::ConcatMatrix(matrix) => {
                canvas.concat(matrix);
            }
            Command::Save => {
                canvas.save();
            }
            Command::Restore => {
                canvas.restore();
            }
            Command::Rect(rect, paint) => {
                canvas.draw_rect(rect, paint);
            }
            Command::Arc(oval, start, sweep, center, paint) => {
                canvas.draw_arc(oval, *start, *sweep, *center, paint);
            }
            Command::Str(str, origin, font, style, paint) => {
                canvas.draw_str(str, *origin, font_set.get(*font, *style), paint);
            }
        }
    }
}

pub trait FontSet {
    fn get(&self, font: Font, style: FontStyle) -> &skia_safe::Font {
        match font {
            Font::Default => self.get_default(style),
        }
    }

    fn get_default(&self, style: FontStyle) -> &skia_safe::Font;
}
