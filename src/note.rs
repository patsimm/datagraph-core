use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum Note {
    C0, CSharp0, D0, DSharp0, E0, F0, FSharp0, G0, GSharp0, A0, ASharp0, B0,
    C1, CSharp1, D1, DSharp1, E1, F1, FSharp1, G1, GSharp1, A1, ASharp1, B1,
    C2, CSharp2, D2, DSharp2, E2, F2, FSharp2, G2, GSharp2, A2, ASharp2, B2,
    C3, CSharp3, D3, DSharp3, E3, F3, FSharp3, G3, GSharp3, A3, ASharp3, B3,
    C4, CSharp4, D4, DSharp4, E4, F4, FSharp4, G4, GSharp4, A4, ASharp4, B4,
    C5, CSharp5, D5, DSharp5, E5, F5, FSharp5, G5, GSharp5, A5, ASharp5, B5,
    C6, CSharp6, D6, DSharp6, E6, F6, FSharp6, G6, GSharp6, A6, ASharp6, B6,
    C7, CSharp7, D7, DSharp7, E7, F7, FSharp7, G7, GSharp7, A7, ASharp7, B7,
    C8, CSharp8, D8, DSharp8, E8, F8, FSharp8, G8, GSharp8, A8, ASharp8, B8,
    C9, CSharp9, D9, DSharp9, E9, F9, FSharp9, G9, GSharp9, A9, ASharp9, B9,
}

#[rustfmt::skip]
const ALL_NOTES: [Note; 120] = [
    Note::C0, Note::CSharp0, Note::D0, Note::DSharp0, Note::E0, Note::F0, Note::FSharp0, Note::G0, Note::GSharp0, Note::A0, Note::ASharp0, Note::B0,
    Note::C1, Note::CSharp1, Note::D1, Note::DSharp1, Note::E1, Note::F1, Note::FSharp1, Note::G1, Note::GSharp1, Note::A1, Note::ASharp1, Note::B1,
    Note::C2, Note::CSharp2, Note::D2, Note::DSharp2, Note::E2, Note::F2, Note::FSharp2, Note::G2, Note::GSharp2, Note::A2, Note::ASharp2, Note::B2,
    Note::C3, Note::CSharp3, Note::D3, Note::DSharp3, Note::E3, Note::F3, Note::FSharp3, Note::G3, Note::GSharp3, Note::A3, Note::ASharp3, Note::B3,
    Note::C4, Note::CSharp4, Note::D4, Note::DSharp4, Note::E4, Note::F4, Note::FSharp4, Note::G4, Note::GSharp4, Note::A4, Note::ASharp4, Note::B4,
    Note::C5, Note::CSharp5, Note::D5, Note::DSharp5, Note::E5, Note::F5, Note::FSharp5, Note::G5, Note::GSharp5, Note::A5, Note::ASharp5, Note::B5,
    Note::C6, Note::CSharp6, Note::D6, Note::DSharp6, Note::E6, Note::F6, Note::FSharp6, Note::G6, Note::GSharp6, Note::A6, Note::ASharp6, Note::B6,
    Note::C7, Note::CSharp7, Note::D7, Note::DSharp7, Note::E7, Note::F7, Note::FSharp7, Note::G7, Note::GSharp7, Note::A7, Note::ASharp7, Note::B7,
    Note::C8, Note::CSharp8, Note::D8, Note::DSharp8, Note::E8, Note::F8, Note::FSharp8, Note::G8, Note::GSharp8, Note::A8, Note::ASharp8, Note::B8,
    Note::C9, Note::CSharp9, Note::D9, Note::DSharp9, Note::E9, Note::F9, Note::FSharp9, Note::G9, Note::GSharp9, Note::A9, Note::ASharp9, Note::B9,
];

impl From<&str> for Note {
    fn from(note_name: &str) -> Self {
        let note_names = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let (name, octave_str) = note_name.split_at(note_name.len() - 1);
        let octave: usize = octave_str
            .parse()
            .expect("Can't parse octave `{octave_str}`");
        let note_index = note_names
            .iter()
            .position(|&n| n == name)
            .expect("Can't parse note name `{name}`");
        let index = octave * 12 + note_index;
        ALL_NOTES[index]
    }
}

impl Note {
    pub fn midi(&self) -> MidiNote {
        MidiNote::from(*self)
    }
}

pub struct MidiNote(u8);

impl MidiNote {
    pub fn to_note(self) -> Option<Note> {
        let index = self.0 as usize - 12;
        if index < ALL_NOTES.len() {
            Some(ALL_NOTES[index])
        } else {
            None
        }
    }
}

impl Deref for MidiNote {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u8> for MidiNote {
    fn from(num: u8) -> Self {
        Self(num)
    }
}

impl From<Note> for MidiNote {
    fn from(note: Note) -> Self {
        let note_num = ALL_NOTES
            .iter()
            .position(|&n| n == note)
            .expect("Can't find note in ALL_NOTES");
        Self(note_num as u8 + 12)
    }
}

#[cfg(test)]
mod tests {
    use crate::frequency::Frequency;

    use super::*;

    #[test]
    fn midi_0_is_frequency_8_18hz() {
        let note = MidiNote(0);
        let result: Frequency = note.into();
        assert_eq!(8.175773, result.hz());
    }

    #[test]
    fn midi_69_is_frequency_440hz() {
        let note = MidiNote(69);
        let result: Frequency = note.into();
        assert_eq!(440.0, result.hz());
    }

    #[test]
    fn midi_127_is_frequency_12543hz() {
        let note: Frequency = MidiNote(127).into();
        assert_eq!(12543.888, note.hz());
    }

    #[test]
    fn midi_60_is_frequency_261hz() {
        let note: MidiNote = 60.into();
        let result: Frequency = note.into();
        assert_eq!(261.62546, result.hz());
    }

    #[test]
    fn note_c4_is_note_60() {
        let note: Note = "C4".into();
        assert_eq!(60, *note.midi());
    }

    #[test]
    fn notename_c0_is_note_12() {
        let note: Note = "C0".into();
        assert_eq!(12, *note.midi());
    }

    #[test]
    fn notename_c4_is_note_60() {
        let note: Note = "C4".into();
        assert_eq!(60, *note.midi());
    }

    #[test]
    fn midi_12_is_c0() {
        let note = MidiNote(12)
            .to_note()
            .expect("Can't convert MIDI note 12 to Note");
        assert_eq!(Note::C0, note);
    }
}
