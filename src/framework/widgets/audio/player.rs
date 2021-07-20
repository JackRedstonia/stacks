use super::{AudioResource, AudioStream};
use crate::prelude::*;

const FFT_SIZE: usize = 512;
type FftInterpolation = [f32; FFT_SIZE];

pub struct AudioPlayer {
    pub layout_size: LayoutSize,
    pub foreground: Paint,
    pub background: Paint,
    pub fft_paint: Paint,
    pub interpolation_factor: f32,
    audio: ResourceUser<AudioResource>,
    path: String,
    sound: Option<AudioStream>,
    seek_preview_percentage: Option<f32>,
    fft: FftInterpolation,
    size: Size,
    play_lock: bool,
}

impl AudioPlayer {
    pub fn new(
        path: &str,
        size: LayoutSize,
        foreground: Paint,
        background: Paint,
        fft: Paint,
    ) -> Wrap<Self> {
        Self {
            layout_size: size,
            size: Size::new_empty(),
            foreground,
            background,
            fft_paint: fft,
            interpolation_factor: 24.0,
            seek_preview_percentage: None,
            audio: ResourceUser::new_none(),
            path: path.to_owned(),
            fft: [0.0; FFT_SIZE],
            sound: None,
            play_lock: false,
        }
        .into()
    }

    pub fn seek_seconds(&mut self, seconds: f64) -> Result<(), ()> {
        if let Some(sound) = &mut self.sound {
            sound.seek(seconds.clamp(0.0, sound.length()?))?;
        }
        Ok(())
    }

    pub fn seek_percentage(&mut self, percentage: f64) -> Result<(), ()> {
        if let Some(sound) = &mut self.sound {
            sound.seek(sound.length()? * percentage.clamp_unit())?;
        }
        Ok(())
    }

    fn pos_percentage_from_x(&self, x: scalar) -> f64 {
        (x as f64 / self.size.width as f64).clamp_unit()
    }

    fn curve_fft_height(height: f32) -> f32 {
        (((height + 0.0251).log10() + 1.6) * 0.16).max(0.0).min(1.0)
    }

    fn refresh_fft(&mut self, factor: f32) -> Option<()> {
        let vis = self.audio.try_access()?.vis().upgrade()?;
        let vis = vis.lock().ok()?;
        let vis = vis.vals();
        assert!((0.0..=1.0).contains(&factor));
        let factor_inv = 1.0 - factor;
        assert_eq!(vis.len(), FFT_SIZE);
        self.fft.iter_mut().zip(vis.iter()).for_each(|(a, b)| {
            *a = *a * factor_inv + b * factor;
        });
        Some(())
    }
}

impl Widget for AudioPlayer {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(resource) = stack.get::<ResourceUser<AudioResource>>() {
            if &self.audio != resource {
                self.audio = resource.clone();
                self.sound = resource
                    .try_access_mut()
                    .unwrap()
                    .new_audio_stream(&self.path);
            }
        } else {
            self.audio = ResourceUser::new_none();
            self.sound = None;
        }
    }

    fn update(&mut self, _state: &mut WidgetState) {
        let factor = (State::last_update_time().as_secs_f32()
            * self.interpolation_factor)
            .min(1.0);
        self.refresh_fft(factor);
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Space) => {
                if let Some(sound) = &mut self.sound {
                    if !self.play_lock {
                        self.play_lock = true;
                        sound.toggle_playing().unwrap();
                    }
                    return true;
                }
                false
            }
            InputEvent::KeyUp(Keycode::Space) => {
                self.play_lock = false;
                true
            }
            InputEvent::MouseUp(MouseButton::Left, pos) => {
                if state.is_focused() {
                    state.release_focus();
                    let pos = self.pos_percentage_from_x(pos.x);
                    self.seek_percentage(pos).expect("Failed to seek sound");
                    self.seek_preview_percentage = None;
                }
                Rect::from_size(self.size).contains(*pos)
            }
            InputEvent::MouseDown(MouseButton::Left, pos) => {
                let c = Rect::from_size(self.size).contains(*pos);
                if c {
                    state.grab_focus();
                    let pos = (pos.x / self.size.width).clamp_unit();
                    self.seek_preview_percentage = Some(pos);
                }
                c
            }
            InputEvent::MouseMove(pos) => {
                if state.is_focused() || state.is_hovered() {
                    let pos = (pos.x / self.size.width).clamp_unit();
                    self.seek_preview_percentage = Some(pos);
                }
                Rect::from_size(self.size).contains(*pos)
            }
            _ => false,
        }
    }

    fn hover_lost(&mut self, _state: &mut WidgetState) {
        self.seek_preview_percentage = None;
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        (self.layout_size, false)
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        // Draw background
        canvas.draw_rect(Rect::from_size(self.size), &self.background);

        // Draw progress bar
        if let Some(i) = &self.sound {
            if let (Ok(position), Ok(length)) = (i.position(), i.length()) {
                let p = (position / length).min(1.0) as f32;
                let foreground =
                    Rect::from_wh(self.size.width * p, self.size.height);
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
            }
        }

        // Draw visualizations
        let fft = &self.fft[..400];
        let width = self.size.width / fft.len() as f32;
        let mut path = skia::Path::new();
        let spacing = width * 0.48;
        let quad_spacing = width * 0.18;
        path.move_to((0.0, self.size.height));
        let last = fft
            .iter()
            .fold((0.0, self.size.height), |(n, prev), i| {
                let height = self.size.height
                    - Self::curve_fft_height(*i) * self.size.height;
                let mid = (height + prev) / 2.0;
                if n != 0.0 {
                    path.quad_to((n - quad_spacing, prev), (n, mid));
                }
                path.quad_to((n + quad_spacing, height), (n + spacing, height));
                path.line_to((n + width - spacing, height));
                (n + width, height)
            })
            .1;
        path.quad_to((self.size.width, last), self.size.bottom_right());
        path.close();
        canvas.draw_path(&path, &self.fft_paint);
    }
}
