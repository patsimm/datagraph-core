use std::{ops::Deref, time::Duration};

use crate::note::MidiNote;

pub struct Frequency(f32);

impl Deref for Frequency {
    type Target = f32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Frequency> for Duration {
    fn from(freq: Frequency) -> Self {
        Duration::from_secs_f32(1.0 / freq.0)
    }
}

impl From<Duration> for Frequency {
    fn from(duration: Duration) -> Self {
        Self(1.0 / duration.as_secs_f32())
    }
}

impl<T: Into<MidiNote>> From<T> for Frequency {
    fn from(value: T) -> Frequency {
        // A4 is 440Hz and is note number 69
        let a4_freq = 440.0;
        let a4_note_num = 69.0;
        let semitone_ratio = 2f32.powf(1.0 / 12.0);
        let note_num = *value.into() as f32;
        let freq = a4_freq * semitone_ratio.powf(note_num - a4_note_num);
        Frequency::from_hz(freq)
    }
}

pub trait FromHz {
    fn from_hz(hz: impl Into<f32>) -> Self;
}

pub trait FromCv {
    fn from_cv(cv: f32) -> Self;
}

pub trait ToCv {
    fn to_cv(&self) -> f32;
}

impl FromHz for std::time::Duration {
    fn from_hz(hz: impl Into<f32>) -> Self {
        std::time::Duration::from_secs_f32(1.0 / hz.into())
    }
}
impl FromHz for Frequency {
    fn from_hz(freq: impl Into<f32>) -> Self {
        Self(freq.into())
    }
}

impl FromCv for Frequency {
    fn from_cv(cv: f32) -> Self {
        // 1V/octave standard, where 0V = C4 (261.63Hz)
        let c4_freq = 261.63;
        let semitone_ratio = 2f32.powf(1.0 / 12.0);
        let freq = c4_freq * semitone_ratio.powf(cv * 12.0);
        Self(freq)
    }
}

impl ToCv for Frequency {
    fn to_cv(&self) -> f32 {
        // 1V/octave standard, where 0V = C4 (261.63Hz)
        let c4_freq = 261.63;
        let semitone_ratio = 2f32.powf(1.0 / 12.0);
        (self.0 / c4_freq).log(semitone_ratio) / 12.0
    }
}
