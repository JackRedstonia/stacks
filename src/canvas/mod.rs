use skia_safe::{Canvas as SkCanvas, Matrix, Paint, Rect};
use skulpin_renderer::skia_safe;

pub struct Canvas {
    commands: Vec<Command>,
}

impl Canvas {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn play(&self, canvas: &mut SkCanvas) {
        for command in &self.commands {
            command.execute(canvas);
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
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
}

pub enum Command {
    ConcatMatrix(Matrix),
    Save,
    Restore,
    Rect(Rect, Paint),
}

impl Command {
    pub fn execute(&self, canvas: &mut SkCanvas) {
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
        }
    }
}
