use crate::prelude::*;
use game::InputEvent;
use skia::{Canvas, Contains, Paint, Rect, Size};
use skulpin_renderer_sdl2::sdl2::{keyboard::Keycode, mouse::MouseButton};

pub struct AudioPlayer {
    pub layout_size: LayoutSize,
    pub foreground: Paint,
    pub background: Paint,
    size: Size,
    music: Music,
}

impl AudioPlayer {
    pub fn new(size: LayoutSize, foreground: Paint, background: Paint) -> Self {
        let music = Music::new("./src/resources/sound.ogg").unwrap();
        Self {
            layout_size: size,
            size: Size::new_empty(),
            foreground,
            background,
            music,
        }
    }
}

impl Widget for AudioPlayer {
    fn update(&mut self, _wrap: &mut WrapState) {
        if let Err(Some(s)) = self.music.update() {
            eprintln!("Music player received error: {}", s);
        }
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Space) => {
                self.music.toggle_playing();
                true
            }
            InputEvent::MouseDown(MouseButton::Left, pos) => {
                let c = Rect::from_size(self.size).contains(*pos);
                if c {
                    self.music.seek_percentage(pos.x / self.size.width);
                }
                c
            }
            InputEvent::MouseMove(pos) => Rect::from_size(self.size).contains(*pos),
            _ => false,
        }
    }

    fn hover(&mut self, _wrap: &mut WrapState) {
        println!("I was hovered!");
    }

    fn hover_lost(&mut self, _wrap: &mut WrapState) {
        println!("I lost hover!");
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.draw_rect(Rect::from_size(self.size), &self.background);
        if let Some(percentage) = self.music.position_percentage() {
            let foreground = Rect::from_wh(self.size.width * percentage, self.size.height);
            canvas.draw_rect(foreground, &self.foreground);
        }
    }
}
