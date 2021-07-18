#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayMode {
    Once,
    Loop,
    Bidirectional,
}

impl PlayMode {
    fn from_allegro_playmode(p: allegro_audio::Playmode) -> Self {
        match p {
            allegro_audio::Playmode::Once => PlayMode::Once,
            allegro_audio::Playmode::Loop => PlayMode::Loop,
            allegro_audio::Playmode::BiDir => PlayMode::Bidirectional,
        }
    }

    fn to_allegro_playmode(self) -> allegro_audio::Playmode {
        match self {
            PlayMode::Once => allegro_audio::Playmode::Once,
            PlayMode::Loop => allegro_audio::Playmode::Loop,
            PlayMode::Bidirectional => allegro_audio::Playmode::BiDir,
        }
    }
}

pub struct AudioStream {
    inner: allegro_audio::AudioStream,
    frag_size: usize,
}

impl AudioStream {
    pub(super) fn from_allegro_stream(
        s: allegro_audio::AudioStream,
        frag_size: usize,
    ) -> Self {
        Self {
            inner: s,
            frag_size,
        }
    }

    pub fn length(&self) -> Result<f64, ()> {
        self.inner.get_length_secs()
    }

    pub fn is_playing(&self) -> bool {
        self.inner.get_playing()
    }

    pub fn set_playing(&self, playing: bool) -> Result<(), ()> {
        self.inner.set_playing(playing)
    }

    pub fn toggle_playing(&self) -> Result<bool, ()> {
        let new_state = !self.is_playing();
        self.set_playing(new_state)?;
        Ok(new_state)
    }

    pub fn drain(&self) {
        self.inner.drain()
    }

    pub fn gain(&self) -> f32 {
        self.inner.get_gain()
    }

    pub fn set_gain(&self, gain: f32) -> Result<(), ()> {
        self.inner.set_gain(gain)
    }

    pub fn pan(&self) -> f32 {
        self.inner.get_pan()
    }

    pub fn set_pan(&self, pan: Option<f32>) -> Result<(), ()> {
        self.inner.set_pan(pan)
    }

    pub fn speed(&self) -> f32 {
        self.inner.get_speed()
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), ()> {
        self.inner.set_speed(speed)
    }

    pub fn play_mode(&self) -> PlayMode {
        PlayMode::from_allegro_playmode(self.inner.get_playmode())
    }

    pub fn set_play_mode(&self, play_mode: PlayMode) -> Result<(), ()> {
        self.inner.set_playmode(play_mode.to_allegro_playmode())
    }

    pub fn position(&self) -> Result<f64, ()> {
        self.inner.get_position_secs()
    }

    pub fn frequency(&self) -> u32 {
        self.inner.get_frequency()
    }

    pub fn fragment_sample_count(&self) -> usize {
        self.frag_size
    }

    pub fn fragment_count(&self) -> u32 {
        self.inner.get_num_fragments()
    }

    pub fn seek(&self, position: f64) -> Result<(), ()> {
        self.inner.seek_secs(position)
    }
}

pub struct Sample {
    pub(super) inner: allegro_audio::Sample,
}

impl Sample {
    pub(super) fn from_allegro_sample(s: allegro_audio::Sample) -> Self {
        Self { inner: s }
    }
}

pub struct SampleInstance {
    inner: allegro_audio::SampleInstance,
}

impl SampleInstance {
    pub(super) fn from_allegro_sample_instance(
        s: allegro_audio::SampleInstance,
    ) -> Self {
        Self { inner: s }
    }

    pub fn length(&self) -> Result<f64, ()> {
        let s = self.inner.get_length()?;
        let f = self.inner.get_frequency()?;
        Ok(s as f64 / f as f64)
    }

    pub fn is_playing(&self) -> Result<bool, ()> {
        self.inner.get_playing()
    }

    pub fn set_playing(&self, playing: bool) -> Result<(), ()> {
        self.inner.set_playing(playing)
    }

    pub fn toggle_playing(&self) -> Result<bool, ()> {
        let new_state = !self.is_playing()?;
        self.set_playing(new_state)?;
        Ok(new_state)
    }

    pub fn gain(&self) -> Result<f32, ()> {
        self.inner.get_gain()
    }

    pub fn set_gain(&self, gain: f32) -> Result<(), ()> {
        self.inner.set_gain(gain)
    }

    pub fn pan(&self) -> Result<f32, ()> {
        self.inner.get_pan()
    }

    pub fn set_pan(&self, pan: Option<f32>) -> Result<(), ()> {
        self.inner.set_pan(pan)
    }

    pub fn speed(&self) -> Result<f32, ()> {
        self.inner.get_speed()
    }

    pub fn set_speed(&self, speed: f32) -> Result<(), ()> {
        self.inner.set_speed(speed)
    }

    pub fn play_mode(&self) -> Result<PlayMode, ()> {
        let p = self.inner.get_playmode()?;
        Ok(PlayMode::from_allegro_playmode(p))
    }

    pub fn set_play_mode(&self, play_mode: PlayMode) -> Result<(), ()> {
        self.inner.set_playmode(play_mode.to_allegro_playmode())
    }

    pub fn position(&self) -> Result<f64, ()> {
        let s = self.inner.get_position()?;
        let f = self.inner.get_frequency()?;
        Ok(s as f64 / f as f64)
    }
}
