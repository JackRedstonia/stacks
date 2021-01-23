use crate::prelude::*;
use soloud::{
    AudioExt, Backend as SoloudBackend, Bus as SoloudBus, Handle as SoloudHandle, Soloud,
    SoloudError, SoloudFlag,
};

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
    pub fn new() -> Result<ResourceHoster<Self>, SoloudError> {
        let mut soloud = Soloud::new(
            SoloudFlag::ClipRoundoff | SoloudFlag::EnableVisualization,
            SoloudBackend::Auto,
            44100,
            860,
            2,
        )?;
        soloud.set_global_volume(2.0);
        let default_bus = SoloudBus::default();
        default_bus.set_visualize_enable(true);
        let default_bus_instance = soloud.play(&default_bus);
        Ok(ResourceHoster::new(Self {
            soloud,
            default_bus: (default_bus, default_bus_instance),
        }))
    }

    pub fn play_sound<T>(&self, sound: &T, bus: Option<AudioBus>) -> SoloudHandle
    where
        T: AudioExt,
    {
        self.play_sound_ex(sound, None, None, None, bus.unwrap_or_default())
    }

    pub fn play_sound_ex<T>(
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

    pub fn play_sound_clocked<T>(&self, time: f64, sound: &T) -> SoloudHandle
    where
        T: AudioExt,
    {
        self.soloud.play_clocked(time, sound)
    }

    pub fn play_sound_clocked_ex<T>(
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

    pub fn play_sound_handle(&mut self, handle: SoloudHandle) {
        self.soloud.set_pause(handle, false);
    }

    pub fn pause_sound_handle(&mut self, handle: SoloudHandle) {
        self.soloud.set_pause(handle, true)
    }

    pub fn set_playing_sound_handle(&mut self, handle: SoloudHandle, playing: bool) {
        self.soloud.set_pause(handle, !playing)
    }

    pub fn toggle_playing_sound_handle(&mut self, handle: SoloudHandle) -> bool {
        let was_paused = self.soloud.pause(handle);
        self.soloud.set_pause(handle, !was_paused);
        was_paused
    }

    pub fn is_playing_sound_handle(&self, handle: SoloudHandle) -> bool {
        // The method is called "pause" but it actually returns a boolean
        // indicating whether the handle is paused for some reason.
        // Lucky we get to abstract it out here so users of the library
        // doesn't get confused.
        !self.soloud.pause(handle)
    }

    pub fn seek_sound_handle(&self, handle: SoloudHandle, seconds: f64) -> Result<(), SoloudError> {
        self.soloud.seek(handle, seconds)
    }

    pub fn playback_position_sound_handle(&self, handle: SoloudHandle) -> f64 {
        self.soloud.stream_position(handle)
    }

    pub fn get_sound_master_fft(&self) -> Vec<f32> {
        self.soloud.calc_fft()
    }

    pub fn set_sound_handle_speed(
        &mut self,
        handle: SoloudHandle,
        speed: f32,
    ) -> Result<(), SoloudError> {
        self.soloud.set_relative_play_speed(handle, speed)
    }

    pub fn get_sound_handle_speed(&self, handle: SoloudHandle) -> f32 {
        self.soloud.relative_play_speed(handle)
    }
}

pub struct Audio<T: Widget> {
    resource: ResourceHoster<AudioResource>,
    inner: Wrap<T>,
}

impl<T: Widget> Audio<T> {
    pub fn new(inner: impl Into<Wrap<T>>) -> Result<Self, SoloudError> {
        FrameworkState::request_load();
        Ok(Self {
            resource: AudioResource::new()?,
            inner: inner.into(),
        })
    }
}

impl<T: Widget> Widget for Audio<T> {
    fn load(&mut self, wrap: &mut WrapState, stack: &mut ResourceStack) {
        stack.push(self.resource.new_user());
        self.inner.load(stack);
        stack.pop::<ResourceUser<AudioResource>>();
    }

    fn update(&mut self, wrap: &mut WrapState) {
        self.inner.update();
    }

    fn input(&mut self, wrap: &mut WrapState, event: &game::InputEvent) -> bool {
        self.inner.input(event)
    }

    fn size(&mut self, wrap: &mut WrapState) -> (LayoutSize, bool) {
        self.inner.size()
    }

    fn set_size(&mut self, wrap: &mut WrapState, size: skia::Size) {
        self.inner.set_size(size);
    }

    fn draw(&mut self, wrap: &mut WrapState, canvas: &mut skia::Canvas) {
        self.inner.draw(canvas);
    }
}

use soloud::{Handle, LoadExt, Wav, WavStream};
use std::path::Path;

pub struct Sound<T>
where
    T: AudioExt,
{
    source: T,
}

impl Sound<Wav> {
    pub fn new_wav(wav: Wav) -> Self {
        Self::from_source(wav)
    }

    pub fn new_wav_from_path<P: AsRef<Path>>(path: P) -> Result<Self, SoloudError> {
        let mut wav = Wav::default();
        wav.load(path.as_ref())?;
        Ok(Self::new_wav(wav))
    }

    pub fn length(&self) -> f64 {
        self.source.length()
    }
}

impl Sound<WavStream> {
    pub fn new_wav_stream(wav_stream: WavStream) -> Self {
        Self::from_source(wav_stream)
    }

    pub fn new_wav_stream_from_path<P: AsRef<Path>>(path: P) -> Result<Self, SoloudError> {
        let mut wav = WavStream::default();
        wav.load(path.as_ref())?;
        Ok(Self::new_wav_stream(wav))
    }

    pub fn length(&self) -> f64 {
        self.source.length()
    }
}

impl<T> Sound<T>
where
    T: AudioExt,
{
    pub fn from_source(source: T) -> Self {
        Self { source }
    }

    pub fn create_instance(
        &self,
        resource: &ResourceUser<AudioResource>,
        bus: Option<AudioBus>,
    ) -> SoundInstance {
        let bus = bus.unwrap_or_default();
        let resource = resource.clone();
        let rsc = resource
            .try_access()
            .expect("Failed to access audio resource");
        let handle = rsc.play_sound_ex(&self.source, None, None, Some(true), bus);
        drop(rsc);
        SoundInstance {
            resource_user: resource,
            handle,
            bus,
        }
    }

    pub fn play_clocked(
        &self,
        resource: &ResourceUser<AudioResource>,
        time: f64,
        bus: Option<AudioBus>,
    ) -> SoundInstance {
        let bus = bus.unwrap_or_default();
        let resource = resource.clone();
        let rsc = resource
            .try_access()
            .expect("Failed to access audio resource");
        let handle = rsc.play_sound_clocked(time, &self.source);
        drop(rsc);
        SoundInstance {
            resource_user: resource,
            handle,
            bus,
        }
    }
}

pub struct SoundInstance {
    resource_user: ResourceUser<AudioResource>,
    handle: Handle,
    bus: AudioBus,
}

impl SoundInstance {
    pub fn bus(&self) -> &AudioBus {
        &self.bus
    }

    pub fn play(&self) {
        if let Some(mut e) = self.resource_user.try_access_mut() {
            e.play_sound_handle(self.handle);
        }
    }

    pub fn pause(&self) {
        if let Some(mut e) = self.resource_user.try_access_mut() {
            e.pause_sound_handle(self.handle);
        }
    }

    pub fn set_playing(&self, playing: bool) {
        if let Some(mut e) = self.resource_user.try_access_mut() {
            e.set_playing_sound_handle(self.handle, playing);
        }
    }

    pub fn toggle_playing(&self) -> bool {
        self.resource_user
            .try_access_mut()
            .map(|mut e| e.toggle_playing_sound_handle(self.handle))
            .unwrap_or(false)
    }

    pub fn is_playing(&self) -> bool {
        self.resource_user
            .try_access()
            .map(|e| e.is_playing_sound_handle(self.handle))
            .unwrap_or(false)
    }

    pub fn seek(&self, seconds: f64) -> Result<(), SoloudError> {
        self.resource_user
            .try_access_mut()
            .map(|e| {
                let position = e.playback_position_sound_handle(self.handle);
                if seconds > position + 2.0 {
                    e.seek_sound_handle(self.handle, 0.0)?;
                }
                e.seek_sound_handle(self.handle, seconds)
            })
            .unwrap_or(Ok(()))
    }

    pub fn position(&self) -> f64 {
        self.resource_user
            .try_access()
            .map(|e| e.playback_position_sound_handle(self.handle))
            .unwrap_or(0.0)
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), SoloudError> {
        self.resource_user
            .try_access_mut()
            .map(|mut e| e.set_sound_handle_speed(self.handle, speed))
            .unwrap_or(Ok(()))
    }

    pub fn speed(&self) -> f32 {
        self.resource_user
            .try_access()
            .map(|e| e.get_sound_handle_speed(self.handle))
            .unwrap_or(1.0)
    }
}
