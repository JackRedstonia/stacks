use std::path::Path;

use super::{AudioBus, AudioResource};
use crate::prelude::*;
use soloud::{
    AudioExt, Bus as SoloudBus, Handle as SoloudHandle, LoadExt, SoloudError, Wav, WavStream,
};

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
        let handle = rsc.play_ex(&self.source, None, None, Some(true), bus);
        drop(rsc);
        SoundInstance {
            resource,
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
        let handle = rsc.play_clocked(time, &self.source);
        drop(rsc);
        SoundInstance {
            resource,
            handle,
            bus,
        }
    }

    pub fn play_clocked_ex(
        &self,
        resource: &ResourceUser<AudioResource>,
        time: f64,
        volume: Option<f32>,
        pan: Option<f32>,
        bus: Option<AudioBus>,
    ) -> SoundInstance {
        let bus = bus.unwrap_or_default();
        let resource = resource.clone();
        let rsc = resource
            .try_access()
            .expect("Failed to access audio resource");
        let handle = rsc.play_clocked_ex(time, &self.source, volume, pan, bus);
        drop(rsc);
        SoundInstance {
            resource,
            handle,
            bus,
        }
    }
}

pub struct SoundInstance {
    resource: ResourceUser<AudioResource>,
    handle: SoloudHandle,
    bus: AudioBus,
}

impl SoundInstance {
    pub fn resource(&self) -> &ResourceUser<AudioResource> {
        &self.resource
    }

    pub fn bus(&self) -> Option<ResourceUsage<SoloudBus>> {
        self.resource
            .try_access()
            .map(|e| e.map(|e| self.bus.to_bus(&*e)))
    }

    pub fn resume(&self) {
        if let Some(mut e) = self.resource.try_access_mut() {
            e.resume(self.handle);
        }
    }

    pub fn pause(&self) {
        if let Some(mut e) = self.resource.try_access_mut() {
            e.pause(self.handle);
        }
    }

    pub fn set_playing(&self, playing: bool) {
        if let Some(mut e) = self.resource.try_access_mut() {
            e.set_playing(self.handle, playing);
        }
    }

    pub fn toggle_playing(&self) -> bool {
        self.resource
            .try_access_mut()
            .map(|mut e| e.toggle_playing(self.handle))
            .unwrap_or(false)
    }

    pub fn is_playing(&self) -> bool {
        self.resource
            .try_access()
            .map(|e| e.is_playing(self.handle))
            .unwrap_or(false)
    }

    pub fn seek(&self, seconds: f64) -> Result<(), SoloudError> {
        self.resource
            .try_access_mut()
            .map(|e| e.seek(self.handle, seconds))
            .unwrap_or(Ok(()))
    }

    pub fn position(&self) -> f64 {
        self.resource
            .try_access()
            .map(|e| e.position(self.handle))
            .unwrap_or(0.0)
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), SoloudError> {
        self.resource
            .try_access_mut()
            .map(|mut e| e.set_speed(self.handle, speed))
            .unwrap_or(Ok(()))
    }

    pub fn speed(&self) -> f32 {
        self.resource
            .try_access()
            .map(|e| e.speed(self.handle))
            .unwrap_or(1.0)
    }

    pub fn set_volume(&self, volume: f32) {
        if let Some(mut resource) = self.resource.try_access_mut() {
            resource.set_volume(self.handle, volume);
        }
    }

    pub fn volume(&self) -> f32 {
        self.resource
            .try_access()
            .map(|e| e.volume(self.handle))
            .unwrap_or(1.0)
    }
}
