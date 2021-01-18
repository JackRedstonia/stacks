use crate::prelude::*;
use audio::{Sound, SoundInstance};
use game::{InputEvent, State};
use skia::{scalar, Canvas, Contains, Paint, Rect, Size};
use skulpin_renderer_sdl2::sdl2::{keyboard::Keycode, mouse::MouseButton};
use soloud::{SoloudError, WavStream};

const FFT_SIZE: usize = 22;
type FftArray = [f32; FFT_SIZE];
type FftPositionArray = [usize; FFT_SIZE];

pub struct AudioPlayer {
    pub layout_size: LayoutSize,
    pub foreground: Paint,
    pub background: Paint,
    pub fft_paint: Paint,
    seek_preview_percentage: Option<f32>,
    size: Size,
    sound: Sound<WavStream>,
    instance: SoundInstance,
    fft: FftArray,
    fft_count: usize,
    interpolated_fft: FftArray,
}

impl AudioPlayer {
    const FFT_POSITIONS: FftPositionArray = [1, 12, 23, 35, 47, 58, 70, 81, 93, 105, 116, 128, 140, 151, 163, 175, 186, 198, 210, 221, 233, 244];
    
    pub fn new(
        size: LayoutSize,
        foreground: Paint,
        background: Paint,
        fft: Paint,
    ) -> Result<Self, SoloudError> {
        let path = "src/resources/sound.ogg";
        let sound = Sound::new_wav_stream_from_path(path)?;
        let instance = sound.create_instance();
        Ok(Self {
            layout_size: size,
            size: Size::new_empty(),
            foreground,
            background,
            fft_paint: fft,
            seek_preview_percentage: None,
            sound,
            instance,
            fft: [0.0; FFT_SIZE],
            fft_count: 0,
            interpolated_fft: [0.0; FFT_SIZE],
        })
    }

    fn pos_percentage_from_x(&self, x: scalar) -> f64 {
        (x as f64 / self.size.width as f64).clamp(0.0, 1.0)
    }

    fn seek_percentage(&self, percentage: f64) -> Result<(), SoloudError> {
        self.instance
            .seek(self.sound.length() * percentage.clamp(0.0, 1.0))
    }

    fn take_fft(&mut self) {
        let fft = State::get_fft();
        let fft_iter = Self::FFT_POSITIONS.iter().map(|e| fft[*e]);
        for (a, b) in self.fft.iter_mut().zip(fft_iter) {
            *a += b;
        }
        self.fft_count += 1;
    }

    fn interpolate_fft(&mut self) {
        let count = self.fft_count as f32;
        self.interpolated_fft.iter_mut().zip(self.fft.iter().map(|f| f / count)).for_each(|(a, b)| {
            let factor = 0.3;
            *a = b * factor + (*a * (1.0 - factor));
        });
    }

    fn clear_fft(&mut self) {
        self.fft = [0.0; FFT_SIZE];
        self.fft_count = 0;
    }
}

impl Widget for AudioPlayer {
    fn update(&mut self, _wrap: &mut WrapState) {
        self.take_fft();
    }

    fn input(&mut self, wrap: &mut WrapState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Space) => {
                self.instance.toggle_playing();
                true
            }
            InputEvent::MouseUp(MouseButton::Left, pos) => {
                if wrap.is_focused() {
                    wrap.release_focus();
                    let pos = self.pos_percentage_from_x(pos.x);
                    self.seek_percentage(pos).expect("Failed to seek sound");
                    self.seek_preview_percentage = None;
                }
                Rect::from_size(self.size).contains(*pos)
            }
            InputEvent::MouseDown(MouseButton::Left, pos) => {
                let c = Rect::from_size(self.size).contains(*pos);
                if c {
                    wrap.grab_focus();
                    let pos = (pos.x / self.size.width).clamp(0.0, 1.0);
                    self.seek_preview_percentage = Some(pos);
                }
                c
            }
            InputEvent::MouseMove(pos) => {
                if wrap.is_focused() || wrap.is_hovered() {
                    let pos = (pos.x / self.size.width).clamp(0.0, 1.0);
                    self.seek_preview_percentage = Some(pos);
                }
                Rect::from_size(self.size).contains(*pos)
            }
            _ => false,
        }
    }

    fn hover_lost(&mut self, _wrap: &mut WrapState) {
        self.seek_preview_percentage = None;
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut Canvas) {
        canvas.draw_rect(Rect::from_size(self.size), &self.background);
        let percentage = (self.instance.position() / self.sound.length()) as f32;
        let foreground = Rect::from_wh(self.size.width * percentage, self.size.height);
        canvas.draw_rect(foreground, &self.foreground);
        if let Some(preview) = self.seek_preview_percentage {
            let center = self.size.width * preview;
            let p = Rect::new(
                (center - 2.0).max(0.0),
                0.0,
                (center + 2.0).min(self.size.width),
                self.size.height,
            );
            canvas.draw_rect(p, &self.background);
        }
        
        self.interpolate_fft();
        let fft = &self.interpolated_fft;
        let width = self.size.width / fft.len() as f32;
        let mut path = skia::Path::new();
        let spacing = width * 0.47;
        let quad_spacing = width * 0.2;
        path.move_to((0.0, self.size.height));
        let last = fft.iter().fold((0.0, self.size.height), |(n, prev), i| {
            let height = self.size.height - (i / 10.0).powf(0.8) * self.size.height;
            let mid = (height + prev) / 2.0;
            path.quad_to((n - quad_spacing, prev), (n, mid));
            path.quad_to((n + quad_spacing, height), (n + spacing, height));
            path.line_to((n + width - spacing, height));
            (n + width, height)
        }).1;
        path.quad_to((self.size.width, last), self.size.bottom_right());
        path.close();
        canvas.draw_path(&path, &self.fft_paint);
        self.clear_fft();
    }
}
