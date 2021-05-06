use std::path::Path;

use super::{AudioBus, AudioResource, Sound, SoundInstance};
use crate::prelude::*;
use soloud::{SoloudError, WavStream};

const FFT_SIZE: usize = 256;
type FftInterpolation = [f32; FFT_SIZE];

pub struct AudioPlayer {
    pub layout_size: LayoutSize,
    pub foreground: Paint,
    pub background: Paint,
    pub fft_paint: Paint,
    sound: Sound<WavStream>,
    instance: Option<SoundInstance>,
    seek_preview_percentage: Option<f32>,
    size: Size,
    fft: FftInterpolation,
    play_lock: bool,
}

impl AudioPlayer {
    pub fn new<P: AsRef<Path>>(
        path: P,
        size: LayoutSize,
        foreground: Paint,
        background: Paint,
        fft: Paint,
    ) -> Result<Wrap<Self>, SoloudError> {
        let sound = Sound::new_wav_stream_from_path(path)?;
        Ok(Self {
            layout_size: size,
            size: Size::new_empty(),
            foreground,
            background,
            fft_paint: fft,
            seek_preview_percentage: None,
            sound,
            instance: None,
            fft: [0.0; FFT_SIZE],
            play_lock: false,
        }
        .into())
    }

    pub fn seek_seconds(&mut self, seconds: f64) -> Result<(), SoloudError> {
        if let Some(instance) = &mut self.instance {
            instance.seek(seconds.clamp(0.0, self.sound.length()))?;
        }
        Ok(())
    }

    pub fn seek_percentage(
        &mut self,
        percentage: f64,
    ) -> Result<(), SoloudError> {
        if let Some(instance) = &mut self.instance {
            instance.seek(self.sound.length() * percentage.clamp_unit())?;
        }
        Ok(())
    }

    fn pos_percentage_from_x(&self, x: scalar) -> f64 {
        (x as f64 / self.size.width as f64).clamp_unit()
    }

    fn refresh_fft(&mut self, factor: f32) {
        assert!((0.0..=1.0).contains(&factor));
        if let Some(instance) = &self.instance {
            if let Some(bus) = instance.bus() {
                let factor_inv = 1.0 - factor;
                // TODO: this is rather unprofessional as it exposes the
                // implementation's API. perhaps fix it... or not?
                let fft = bus.calc_fft();
                assert_eq!(fft.len(), FFT_SIZE);
                self.fft.iter_mut().zip(fft.iter()).for_each(|(a, b)| {
                    *a = *a * factor_inv + b * factor;
                });
            }
        }
    }

    fn curve_fft_height(height: f32) -> f32 {
        let clipoff_factor = 1.25;
        let clipoff_factor_inv = 1.0 / clipoff_factor;

        let lifted = height * clipoff_factor;
        let clipped = if lifted > 1.0 {
            lifted.powf(0.5)
        } else {
            lifted
        };

        (clipoff_factor_inv * clipped).min(1.0)
    }
}

impl Widget for AudioPlayer {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        if let Some(resource) = stack.get::<ResourceUser<AudioResource>>() {
            if !self
                .instance
                .as_ref()
                .map(|e| e.resource() == resource)
                .unwrap_or(false)
            {
                let bus = AudioBus::Default;
                let mut instance =
                    self.sound.create_instance(resource, Some(bus));
                instance.set_auto_stop(false);
                self.instance = Some(instance);
            }
        }
    }

    fn update(&mut self, _state: &mut WidgetState) {
        let factor = (State::last_update_time().as_secs_f32() * 16.0).min(1.0);
        self.refresh_fft(factor);
    }

    fn input(&mut self, state: &mut WidgetState, event: &InputEvent) -> bool {
        match event {
            InputEvent::KeyDown(Keycode::Space) => {
                if let Some(instance) = &mut self.instance {
                    if !self.play_lock {
                        self.play_lock = true;
                        instance.toggle_playing();
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
        if let Some(i) = &self.instance {
            let p = (i.position() / self.sound.length()).min(1.0) as f32;
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

        // Draw visualizations
        let fft = &self.fft[..180];
        let width = self.size.width / fft.len() as f32;
        let mut path = skia::Path::new();
        let spacing = width * 0.48;
        let quad_spacing = width * 0.18;
        path.move_to((0.0, self.size.height));
        let last = fft
            .iter()
            .fold((0.0, self.size.height), |(n, prev), i| {
                let height = self.size.height
                    - Self::curve_fft_height(i / 16.0) * self.size.height;
                let mid = (height + prev) / 2.0;
                path.quad_to((n - quad_spacing, prev), (n, mid));
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
