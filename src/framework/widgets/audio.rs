mod player;
mod sound;

pub use player::AudioPlayer;
pub use sound::{Sound, SoundInstance};

use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::prelude::*;
use soloud::{
    AudioExt, Backend as SoloudBackend, Bus as SoloudBus, Handle as SoloudHandle, Soloud,
    SoloudError, SoloudFlag,
};

#[derive(Debug)]
pub enum AudioError {
    Soloud(SoloudError),
}

impl Display for AudioError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Self::Soloud(err) => err.fmt(f),
        }
    }
}

impl StdError for AudioError {}

impl From<SoloudError> for AudioError {
    fn from(err: SoloudError) -> Self {
        Self::Soloud(err)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum AudioBus {
    Default,
}

impl AudioBus {
    pub fn to_bus(self, resource: &AudioResource) -> &SoloudBus {
        match self {
            AudioBus::Default => &resource.default_bus.0,
        }
    }
}

impl Default for AudioBus {
    fn default() -> Self {
        Self::Default
    }
}

pub struct AudioResource {
    soloud: Soloud,
    default_bus: (SoloudBus, SoloudHandle),
}

impl AudioResource {
    pub fn new() -> Result<ResourceHoster<Self>, AudioError> {
        let mut soloud = Soloud::new(
            SoloudFlag::ClipRoundoff | SoloudFlag::EnableVisualization,
            SoloudBackend::Auto,
            44100,
            860,
            2,
        )?;
        soloud.set_post_clip_scaler(2.0);
        let default_bus = SoloudBus::default();
        default_bus.set_visualize_enable(true);
        let default_bus_instance = soloud.play(&default_bus);
        Ok(ResourceHoster::new(Self {
            soloud,
            default_bus: (default_bus, default_bus_instance),
        }))
    }

    pub fn play<T>(&self, sound: &T, bus: Option<AudioBus>) -> SoloudHandle
    where
        T: AudioExt,
    {
        self.play_ex(sound, None, None, None, bus.unwrap_or_default())
    }

    pub fn play_ex<T>(
        &self,
        sound: &T,
        volume: Option<f32>,
        pan: Option<f32>,
        paused: Option<bool>,
        bus: AudioBus,
    ) -> SoloudHandle
    where
        T: AudioExt,
    {
        bus.to_bus(&self).play_ex(
            sound,
            volume.unwrap_or(1.0),
            pan.unwrap_or(0.0),
            paused.unwrap_or(false),
        )
    }

    pub fn play_clocked<T>(&self, time: f64, sound: &T) -> SoloudHandle
    where
        T: AudioExt,
    {
        self.soloud.play_clocked(time, sound)
    }

    pub fn play_clocked_ex<T>(
        &self,
        time: f64,
        sound: &T,
        volume: Option<f32>,
        pan: Option<f32>,
        bus: AudioBus,
    ) -> SoloudHandle
    where
        T: AudioExt,
    {
        bus.to_bus(&self)
            .play_clocked_ex(time, sound, volume.unwrap_or(1.0), pan.unwrap_or(0.0))
    }

    pub fn resume(&mut self, handle: SoloudHandle) {
        self.soloud.set_pause(handle, false);
    }

    pub fn pause(&mut self, handle: SoloudHandle) {
        self.soloud.set_pause(handle, true)
    }

    pub fn set_playing(&mut self, handle: SoloudHandle, playing: bool) {
        self.soloud.set_pause(handle, !playing)
    }

    pub fn toggle_playing(&mut self, handle: SoloudHandle) -> bool {
        let was_paused = self.soloud.pause(handle);
        self.soloud.set_pause(handle, !was_paused);
        was_paused
    }

    pub fn is_playing(&self, handle: SoloudHandle) -> bool {
        // The method is called "pause" but it actually returns a boolean
        // indicating whether the handle is paused for some reason.
        // Lucky we get to abstract it out here so users of the library
        // doesn't get confused.
        !self.soloud.pause(handle)
    }

    pub fn seek(&self, handle: SoloudHandle, seconds: f64) -> Result<(), SoloudError> {
        self.soloud.seek(handle, seconds)
    }

    pub fn position(&self, handle: SoloudHandle) -> f64 {
        self.soloud.stream_position(handle)
    }

    pub fn master_fft(&self) -> Vec<f32> {
        self.soloud.calc_fft()
    }

    pub fn set_speed(&mut self, handle: SoloudHandle, speed: f32) -> Result<(), SoloudError> {
        self.soloud.set_relative_play_speed(handle, speed)
    }

    pub fn speed(&self, handle: SoloudHandle) -> f32 {
        self.soloud.relative_play_speed(handle)
    }

    pub fn set_volume(&mut self, handle: SoloudHandle, volume: f32) {
        self.soloud.set_volume(handle, volume);
    }

    pub fn volume(&self, handle: SoloudHandle) -> f32 {
        self.soloud.volume(handle)
    }

    pub fn set_pan(&mut self, handle: SoloudHandle, pan: f32) {
        self.soloud.set_pan(handle, pan);
    }

    pub fn pan(&self, handle: SoloudHandle) -> f32 {
        self.soloud.pan(handle)
    }
}

pub struct Audio<T: Widget> {
    resource: ResourceHoster<AudioResource>,
    inner: Wrap<T>,
}

impl<T: Widget> Audio<T> {
    pub fn new(inner: impl Into<Wrap<T>>) -> Result<Self, AudioError> {
        FrameworkState::request_load();
        Ok(Self {
            resource: AudioResource::new()?,
            inner: inner.into(),
        })
    }
}

impl<T: Widget> Widget for Audio<T> {
    fn load(&mut self, _wrap: &mut WrapState, stack: &mut ResourceStack) {
        stack.push(self.resource.new_user());
        self.inner.load(stack);
        stack.pop::<ResourceUser<AudioResource>>();
    }

    fn update(&mut self, _wrap: &mut WrapState) {
        self.inner.update();
    }

    fn input(&mut self, _wrap: &mut WrapState, event: &game::InputEvent) -> bool {
        self.inner.input(event)
    }

    fn size(&mut self, _wrap: &mut WrapState) -> (LayoutSize, bool) {
        self.inner.size()
    }

    fn set_size(&mut self, _wrap: &mut WrapState, size: skia::Size) {
        self.inner.set_size(size);
    }

    fn draw(&mut self, _wrap: &mut WrapState, canvas: &mut skia::Canvas) {
        self.inner.draw(canvas);
    }
}
