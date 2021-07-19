mod player;
mod sound;

pub use player::AudioPlayer;
pub use sound::{AudioStream, Sample, SampleInstance};

use allegro_audio::{
    AttachToMixer, AudioDepth, ChannelConf, MixerLike, PostProcessCallback,
};
use rustfft::{num_complex::Complex, FftPlanner};

use std::convert::TryInto;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex, Weak};

use crate::prelude::*;

#[derive(Debug)]
pub enum AudioError {
    CoreInitError(String),
    AudioAddonInitError(String),
    AcodecAddonInitError(String),
    SinkInitError(String),
}

impl Display for AudioError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            AudioError::CoreInitError(s) => {
                write!(f, "Initializing Allegro core failed ({})", s)
            }
            AudioError::AudioAddonInitError(s) => write!(
                f,
                "Initializing Allegro audio playback addon failed ({})",
                s
            ),
            AudioError::AcodecAddonInitError(s) => write!(
                f,
                "Initializing Allegro audio codec addon failed ({})",
                s
            ),
            AudioError::SinkInitError(s) => {
                write!(f, "Creating Allegro audio sink failed ({})", s)
            }
        }
    }
}

impl StdError for AudioError {}

pub struct Visuals {
    vals: [Complex<f32>; 1024],
    vals_real: [f32; 512],
}

impl Visuals {
    pub fn vals(&self) -> &[f32; 512] {
        &self.vals_real
    }
}

struct VisualsCallback {
    chan_count: usize,
    vis: Weak<Mutex<Visuals>>,
}

impl PostProcessCallback for VisualsCallback {
    fn process(&mut self, data: &mut [u8], num_samples: u32) {
        assert_eq!(data.len() / num_samples as usize / self.chan_count, 4);
        let fl = data.chunks(4).map(|chunk| {
            let chunk: [u8; 4] = chunk.try_into().unwrap();
            let val: f32 = unsafe { std::mem::transmute(chunk) };
            val
        });
        if let Some(mut vis) =
            self.vis.upgrade().as_ref().and_then(|a| a.lock().ok())
        {
            let vis = vis.deref_mut();
            for (a, b) in vis.vals.iter_mut().zip(fl.step_by(self.chan_count)) {
                *a = Complex::new(b, 0.0);
            }
            let mut fp = FftPlanner::new();
            let fft = fp.plan_fft_forward(1024);
            fft.process(&mut vis.vals);
            for (a, b) in vis.vals.iter().zip(vis.vals_real.iter_mut()) {
                *b = a.re * a.re + a.im * a.im;
            }
        }
    }
}

pub struct AudioResource {
    sink: allegro_audio::Sink,
    audio: allegro_audio::AudioAddon,
    visuals: Arc<Mutex<Visuals>>,
}

impl AudioResource {
    pub fn new() -> Result<ResourceHoster<Self>, AudioError> {
        let allegro =
            allegro::Core::init().map_err(|s| AudioError::CoreInitError(s))?;
        let audio = allegro_audio::AudioAddon::init(&allegro)
            .map_err(|s| AudioError::AudioAddonInitError(s))?;
        allegro_acodec::AcodecAddon::init(&audio)
            .map_err(|s| AudioError::AcodecAddonInitError(s))?;
        let mixer = allegro_audio::Mixer::new(&audio).map_err(|_| {
            AudioError::SinkInitError("Could not initialize mixer".to_owned())
        })?;
        assert!(matches!(mixer.get_channels(), ChannelConf::Conf2));
        assert!(matches!(mixer.get_depth(), AudioDepth::F32));
        let mut sink = allegro_audio::Sink::new_with_mixer(
            44100,
            AudioDepth::I16,
            ChannelConf::Conf2,
            mixer,
        )
        .map_err(|s| AudioError::SinkInitError(s))?;

        let vis = Visuals {
            vals: [Complex::default(); 1024],
            vals_real: [0.0; 512],
        };
        let vis = Arc::new(Mutex::new(vis));
        let callback = Box::new(VisualsCallback {
            chan_count: 2,
            vis: Arc::downgrade(&vis),
        });
        sink.set_postprocess_callback(Some(callback)).unwrap(); // TODO: use .?

        Ok(ResourceHoster::new(Self {
            sink,
            audio,
            visuals: vis,
        }))
    }

    pub fn new_audio_stream(&mut self, path: &str) -> Option<AudioStream> {
        let f = 512;
        let mut s = allegro_audio::AudioStream::load_custom(
            &self.audio,
            path,
            4,
            f as _,
        )
        .ok()?;
        s.set_playing(false).ok()?;
        s.attach(&mut self.sink).ok()?;
        Some(AudioStream::from_allegro_stream(s, f))
    }

    pub fn new_sample(&self, path: &str) -> Option<Sample> {
        let s = allegro_audio::Sample::load(&self.audio, path).ok()?;
        Some(Sample::from_allegro_sample(s))
    }

    pub fn new_sample_instance(
        &mut self,
        sample: &Sample,
    ) -> Option<SampleInstance> {
        let mut s = sample.inner.create_instance().ok()?;
        s.attach(&mut self.sink).ok()?;
        Some(SampleInstance::from_allegro_sample_instance(s))
    }

    pub fn vis(&self) -> Weak<Mutex<Visuals>> {
        Arc::downgrade(&self.visuals)
    }
}

pub struct Audio<T: Widget + ?Sized> {
    child: Wrap<T>,
    resource: ResourceHoster<AudioResource>,
}

impl<T: Widget + ?Sized> Audio<T> {
    pub fn new(child: Wrap<T>) -> Result<Wrap<Self>, AudioError> {
        FrameworkState::request_load();
        Ok(Self {
            child,
            resource: AudioResource::new()?,
        }
        .into())
    }
}

impl<T: Widget + ?Sized> Widget for Audio<T> {
    fn load(&mut self, _state: &mut WidgetState, stack: &mut ResourceStack) {
        stack.push(self.resource.new_user());
        self.child.load(stack);
        stack.pop::<ResourceUser<AudioResource>>();
    }

    fn update(&mut self, _state: &mut WidgetState) {
        self.child.update();
    }

    fn input(&mut self, _state: &mut WidgetState, event: &InputEvent) -> bool {
        self.child.input(event)
    }

    fn size(&mut self, _state: &mut WidgetState) -> (LayoutSize, bool) {
        self.child.size()
    }

    fn set_size(&mut self, _state: &mut WidgetState, size: Size) {
        self.child.set_size(size);
    }

    fn draw(&mut self, _state: &mut WidgetState, canvas: &mut Canvas) {
        self.child.draw(canvas);
    }
}
