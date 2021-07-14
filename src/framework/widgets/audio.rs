mod player;
mod sound;

use allegro_audio::AttachToMixer;
pub use player::AudioPlayer;
pub use sound::{AudioStream, Sample, SampleInstance};

use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

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

pub struct AudioResource {
    sink: allegro_audio::Sink,
    audio: allegro_audio::AudioAddon,
}

impl AudioResource {
    pub fn new() -> Result<ResourceHoster<Self>, AudioError> {
        let allegro =
            allegro::Core::init().map_err(|s| AudioError::CoreInitError(s))?;
        let audio = allegro_audio::AudioAddon::init(&allegro)
            .map_err(|s| AudioError::AudioAddonInitError(s))?;
        allegro_acodec::AcodecAddon::init(&audio)
            .map_err(|s| AudioError::AcodecAddonInitError(s))?;
        let sink = allegro_audio::Sink::new(&audio)
            .map_err(|s| AudioError::SinkInitError(s))?;

        Ok(ResourceHoster::new(Self { sink, audio }))
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
