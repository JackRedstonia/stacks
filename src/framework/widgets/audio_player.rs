use crate::prelude::*;
use game::InputEvent;
use gstreamer::{prelude::*, ClockTime, Element, Format, SeekFlags, State as GstState};
use skia::{Canvas, Contains, Paint, Rect as SkRect, Size};
use skulpin_renderer_sdl2::sdl2::{keyboard::Keycode, mouse::MouseButton};

pub struct AudioPlayer {
    pub layout_size: LayoutSize,
    pub foreground: Paint,
    pub background: Paint,
    size: Size,
    player: Element,
    last_percentage: f32,
}

impl AudioPlayer {
    pub fn new(size: LayoutSize, foreground: Paint, background: Paint) -> Self {
        let player = gstreamer::ElementFactory::make("playbin", Some("player"))
            .expect("Failed to create GStreamer playbin element");
        let path = std::path::Path::new("./src/resources/sound.ogg")
            .canonicalize()
            .unwrap();
        player
            .set_property("uri", &("file://".to_owned() + path.to_str().unwrap()))
            .unwrap();
        player.set_property("volume", &0.6).unwrap();
        player.set_state(GstState::Paused).unwrap();

        Self {
            layout_size: size,
            size: Size::new_empty(),
            foreground,
            background,
            player,
            last_percentage: 0.0,
        }
    }

    fn position(&self) -> Option<u64> {
        self.player
            .query_position::<ClockTime>()
            .map(|p| p.mseconds())
            .flatten()
    }

    fn duration(&self) -> Option<u64> {
        self.player
            .query_duration::<ClockTime>()
            .map(|p| p.mseconds())
            .flatten()
    }

    fn seek(&self, ms: u64) {
        if self.player.get_state(ClockTime::from_mseconds(1)).1 == GstState::Playing {
            let pos = ClockTime::from_mseconds(ms);
            let mut seek_query = gstreamer::query::Seeking::new(Format::Time);
            if self.player.query(&mut seek_query) {
                let seekable = gstreamer::query::Seeking::get_result(&seek_query).0;
                if seekable {
                    let _ = self
                        .player
                        .seek_simple(SeekFlags::FLUSH | SeekFlags::KEY_UNIT, pos);
                }
            }
        }
    }
}

impl Widget for AudioPlayer {
    fn input(&mut self, _wrap: &mut WrapState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Space) => {
                match self.player.get_state(ClockTime::from_mseconds(1)).1 {
                    GstState::Playing => {
                        self.player.set_state(GstState::Paused).unwrap();
                    }
                    GstState::Paused | GstState::Null => {
                        self.player.set_state(GstState::Playing).unwrap();
                    }
                    _ => {}
                };
                true
            }
            InputEvent::MouseDown(MouseButton::Left, pos) => {
                let c = SkRect::from_size(self.size).contains(*pos);
                if c {
                    if let Some(p) = self.duration() {
                        let p = p as f32 * (pos.x / self.size.width);
                        self.seek(p as _);
                    }
                }
                c
            }
            _ => false,
        }
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.draw_rect(SkRect::from_size(self.size), &self.background);
        if let Some(duration) = self.duration() {
            if let Some(position) = self.position() {
                self.last_percentage = position as f32 / duration as f32;
            }
        }
        let width = self.size.width * self.last_percentage;
        let foreground = SkRect::from_wh(width, self.size.height);
        canvas.draw_rect(foreground, &self.foreground);
    }
}
