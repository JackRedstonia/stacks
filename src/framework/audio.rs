use std::path::Path;

use soloud::{AudioExt, Handle, LoadExt, SoloudError, Wav, WavStream};

use crate::game::{AudioBus, State};

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

    pub fn create_instance(&self, bus: Option<AudioBus>) -> SoundInstance {
        let bus = bus.unwrap_or_default();
        SoundInstance {
            handle: State::play_sound_ex(&self.source, None, None, Some(true), bus),
            bus,
        }
    }

    pub fn play_clocked(&self, time: f64, bus: Option<AudioBus>) -> SoundInstance {
        let bus = bus.unwrap_or_default();
        SoundInstance {
            handle: State::play_sound_clocked(time, &self.source),
            bus,
        }
    }
}

pub struct SoundInstance {
    handle: Handle,
    bus: AudioBus,
}

impl SoundInstance {
    pub fn bus(&self) -> &AudioBus {
        &self.bus
    }

    pub fn play(&self) {
        State::play_sound_handle(self.handle);
    }

    pub fn pause(&self) {
        State::pause_sound_handle(self.handle);
    }

    pub fn set_playing(&self, playing: bool) {
        State::set_playing_sound_handle(self.handle, playing)
    }

    pub fn toggle_playing(&self) -> bool {
        State::toggle_playing_sound_handle(self.handle)
    }

    pub fn is_playing(&self) -> bool {
        State::is_playing_sound_handle(self.handle)
    }

    pub fn seek(&self, seconds: f64) -> Result<(), SoloudError> {
        if seconds > self.position() + 2.0 {
            State::seek_sound_handle(self.handle, 0.0)?;
        }
        State::seek_sound_handle(self.handle, seconds)
    }

    pub fn position(&self) -> f64 {
        State::playback_position_sound_handle(self.handle)
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), SoloudError> {
        State::set_sound_handle_speed(self.handle, speed)
    }

    pub fn speed(&self) -> f32 {
        State::get_sound_handle_speed(self.handle)
    }
}
