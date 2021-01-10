use std::io::Error as IoError;
use std::path::Path;
use std::time::Duration;
use std::{cell::RefCell, convert::TryInto};

use gstreamer::{
    glib::{BoolError, FlagsClass},
    prelude::*,
    query::Seeking,
    ClockTime, Element, Format, MessageView, SeekFlags, State as GstState, StateChangeError,
};

pub struct Music {
    player: Element,
    duration: Option<Duration>,
    last_position: RefCell<Duration>,
}

impl Music {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, MusicCreateError> {
        let path = path.as_ref().canonicalize()?;
        let path = "file://".to_owned() + path.to_str().ok_or(MusicCreateError::InvalidPath)?;

        let player = gstreamer::ElementFactory::make("playbin", None)?;
        // Since the file may contain more than just the audio stream,
        // we want to ignore everything but the audio.
        // Making flags through GStreamer Rust bindings is sort of clumsy,
        // it would seem...
        let flags = player.get_property("flags")?;
        let flags_class = FlagsClass::new(flags.type_()).unwrap();
        let flags = flags_class
            .builder_with_value(flags)
            .unwrap()
            .unset_by_nick("text")
            .unset_by_nick("video")
            .build()
            .unwrap();
        player.set_property("flags", &flags)?;
        player.set_property("uri", &path)?;

        player.set_state(GstState::Paused)?;

        Ok(Self {
            player,
            duration: None,
            last_position: RefCell::new(Duration::ZERO),
        })
    }

    pub fn update(&mut self) -> Result<(), Option<String>> {
        if let Some(bus) = self.player.get_bus() {
            if let Some(msg) = bus.pop() {
                match msg.view() {
                    MessageView::StateChanged(_state_changed) => {
                        // TODO: we can actually check for seekability, state,
                        // and other things here.
                    }
                    MessageView::Error(err) => {
                        return Err(err.get_debug());
                    }
                    MessageView::DurationChanged(..) => {
                        // The duration was updated, invalidate old duration
                        self.duration = None;
                    }
                    _ => {}
                }
            }
        }

        if self.duration.is_none() {
            self.duration = self.get_duration();
        }

        Ok(())
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    pub fn duration_f32(&self) -> Option<f32> {
        self.duration().map(|d| d.as_secs_f32())
    }

    pub fn position(&self) -> Duration {
        let pos = match self.player.query_position::<ClockTime>() {
            Some(p) => match p.try_into() {
                Ok(p) => p,
                Err(_) => return *self.last_position.borrow(),
            },
            None => return *self.last_position.borrow(),
        };
        *self.last_position.borrow_mut() = pos;
        pos
    }

    pub fn position_f32(&self) -> f32 {
        self.position().as_secs_f32()
    }

    pub fn position_percentage(&self) -> Option<f32> {
        self.duration_f32().map(|d| self.position_f32() / d)
    }

    pub fn seek(&self, position: Duration) {
        // TODO: check if a check on is_playing is needed
        if self.is_seekable() {
            let position: ClockTime = position.into();
            let seek_flags = SeekFlags::FLUSH | SeekFlags::TRICKMODE_KEY_UNITS;
            let _ = self.player.seek_simple(seek_flags, position);
        }
    }

    pub fn seek_f32(&self, position: f32) {
        self.seek(Duration::from_secs_f32(position));
    }

    pub fn seek_percentage(&self, position: f32) {
        if let Some(duration) = self.duration_f32() {
            self.seek_f32(duration * position);
        }
    }

    pub fn is_playing(&self) -> bool {
        self.get_state() == GstState::Playing
    }

    pub fn play(&self) {
        let _ = self.player.set_state(GstState::Playing);
    }

    pub fn pause(&self) {
        let _ = self.player.set_state(GstState::Paused);
    }

    pub fn toggle_playing(&self) {
        let _ = self.player.set_state(match self.get_state() {
            GstState::Playing => GstState::Paused,
            GstState::Paused | GstState::Null => GstState::Playing,
            _ => return,
        });
    }

    fn get_state(&self) -> GstState {
        self.player.get_state(ClockTime::from_mseconds(1)).1
    }

    fn is_seekable(&self) -> bool {
        let mut query = Seeking::new(Format::Time);
        self.player.query(&mut query) && Seeking::get_result(&query).0
    }

    fn get_duration(&self) -> Option<Duration> {
        self.player
            .query_duration::<ClockTime>()
            .map(|e| e.try_into().ok())
            .flatten()
    }
}

impl Drop for Music {
    fn drop(&mut self) {
        let _ = self.player.set_state(GstState::Null);
    }
}

#[derive(Debug)]
pub enum MusicCreateError {
    GStreamerError,
    IoError(IoError),
    InvalidPath,
}

impl From<IoError> for MusicCreateError {
    fn from(err: IoError) -> Self {
        Self::IoError(err)
    }
}

impl From<BoolError> for MusicCreateError {
    fn from(_: BoolError) -> Self {
        Self::GStreamerError
    }
}

impl From<StateChangeError> for MusicCreateError {
    fn from(_: StateChangeError) -> Self {
        Self::GStreamerError
    }
}
