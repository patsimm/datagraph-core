use std::time::Duration;

use super::note::MidiNote;

pub struct Frequency(f32);

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

pub trait FromBpm {
    fn from_bpm(bpm: impl Into<f32>) -> Self;
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
        // 1V/octave standard, 0V = C4; derive c4_freq from A4=440 to match MIDI conversion
        let semitone_ratio = 2f32.powf(1.0 / 12.0);
        let c4_freq = 440.0 * semitone_ratio.powf(60.0 - 69.0);
        let freq = c4_freq * semitone_ratio.powf(cv * 12.0);
        Self(freq)
    }
}

impl FromBpm for Frequency {
    fn from_bpm(bpm: impl Into<f32>) -> Self {
        let bpm = bpm.into();
        Self(bpm / 60.0)
    }
}

impl ToCv for Frequency {
    fn to_cv(&self) -> f32 {
        // 1V/octave standard, 0V = C4; derive c4_freq from A4=440 to match MIDI conversion
        let semitone_ratio = 2f32.powf(1.0 / 12.0);
        let c4_freq = 440.0 * semitone_ratio.powf(60.0 - 69.0);
        (self.0 / c4_freq).log(semitone_ratio) / 12.0
    }
}

impl Frequency {
    pub fn hz(&self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Frequency, FromBpm, ToCv};
    use crate::dsp::Note;

    #[test]
    fn bpm120_is_0_32cv() {
        let freq = Frequency::from_bpm(120.0);
        assert_eq!(-7.0313535, freq.to_cv());
    }

    #[test]
    fn midi60_is_cv0() {
        let freq: Frequency = Note::C4.into();
        assert_eq!(0.0, freq.to_cv());
    }
}
