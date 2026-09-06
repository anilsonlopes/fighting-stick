use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub const APP_DIR: &str = "FightingStickMidi";
pub const DEFAULT_VELOCITY: u8 = 100;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PhysicalInput {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Button(u8),
    AxisNegative(u8),
    AxisPositive(u8),
}

impl std::fmt::Display for PhysicalInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DPadUp => write!(f, "Direção cima"),
            Self::DPadDown => write!(f, "Direção baixo"),
            Self::DPadLeft => write!(f, "Direção esquerda"),
            Self::DPadRight => write!(f, "Direção direita"),
            Self::Button(n) => write!(f, "Botão {n}"),
            Self::AxisNegative(n) => write!(f, "Eixo {n} −"),
            Self::AxisPositive(n) => write!(f, "Eixo {n} +"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assignment {
    pub input: PhysicalInput,
    pub note: u8,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub controller_guid: String,
    pub controller_name: String,
    pub midi_port: Option<String>,
    pub velocity: u8,
    pub assignments: Vec<Assignment>,
}

impl Profile {
    pub fn default_for(guid: impl Into<String>, name: impl Into<String>) -> Self {
        let inputs = [
            PhysicalInput::DPadUp,
            PhysicalInput::DPadDown,
            PhysicalInput::DPadLeft,
            PhysicalInput::DPadRight,
            PhysicalInput::Button(0),
            PhysicalInput::Button(1),
            PhysicalInput::Button(2),
            PhysicalInput::Button(3),
            PhysicalInput::Button(4),
            PhysicalInput::Button(5),
            PhysicalInput::Button(6),
            PhysicalInput::Button(7),
        ];
        let assignments = inputs
            .into_iter()
            .enumerate()
            .map(|(i, input)| Assignment {
                input,
                note: 36 + i as u8,
                label: note_name(36 + i as u8),
            })
            .collect();
        Self {
            controller_guid: guid.into(),
            controller_name: name.into(),
            midi_port: None,
            velocity: DEFAULT_VELOCITY,
            assignments,
        }
    }

    pub fn validate(&self) -> Result<(), ProfileError> {
        if !(1..=127).contains(&self.velocity) {
            return Err(ProfileError::Velocity(self.velocity));
        }
        let mut notes = BTreeSet::new();
        let mut inputs = BTreeSet::new();
        for a in &self.assignments {
            if a.note > 127 {
                return Err(ProfileError::Note(a.note));
            }
            if !notes.insert(a.note) {
                return Err(ProfileError::DuplicateNote(a.note));
            }
            if !inputs.insert(a.input.clone()) {
                return Err(ProfileError::DuplicateInput(a.input.clone()));
            }
        }
        Ok(())
    }

    pub fn learn(
        &mut self,
        row: usize,
        input: PhysicalInput,
        note: u8,
    ) -> Result<(), ProfileError> {
        if row >= self.assignments.len() {
            return Err(ProfileError::MissingRow(row));
        }
        if note > 127 {
            return Err(ProfileError::Note(note));
        }
        if self
            .assignments
            .iter()
            .enumerate()
            .any(|(i, a)| i != row && a.note == note)
        {
            return Err(ProfileError::DuplicateNote(note));
        }
        // If the physical control is already assigned, swap it with the target
        // row. This lets Learn relocate SDL buttons such as Start/Back without
        // creating duplicate inputs or leaving an unreachable mapping behind.
        if let Some(other_row) = self
            .assignments
            .iter()
            .enumerate()
            .find_map(|(i, a)| (i != row && a.input == input).then_some(i))
        {
            let previous_target = self.assignments[row].input.clone();
            self.assignments[other_row].input = previous_target;
        }
        self.assignments[row].input = input;
        self.assignments[row].note = note;
        self.assignments[row].label = note_name(note);
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProfileError {
    #[error("nota MIDI inválida: {0}")]
    Note(u8),
    #[error("velocidade deve estar entre 1 e 127: {0}")]
    Velocity(u8),
    #[error("a nota MIDI {0} já está em uso")]
    DuplicateNote(u8),
    #[error("a entrada {0} já está em uso")]
    DuplicateInput(PhysicalInput),
    #[error("linha de mapeamento inexistente: {0}")]
    MissingRow(usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MidiCommand {
    NoteOn { note: u8, velocity: u8 },
    NoteOff { note: u8 },
    AllNotesOff,
}

#[derive(Default)]
pub struct MidiState {
    active: BTreeMap<PhysicalInput, u8>,
}

impl MidiState {
    pub fn input(
        &mut self,
        profile: &Profile,
        input: PhysicalInput,
        pressed: bool,
    ) -> Vec<MidiCommand> {
        if pressed {
            if self.active.contains_key(&input) {
                return vec![];
            }
            if let Some(a) = profile.assignments.iter().find(|a| a.input == input) {
                self.active.insert(input, a.note);
                return vec![MidiCommand::NoteOn {
                    note: a.note,
                    velocity: profile.velocity,
                }];
            }
        } else if let Some(note) = self.active.remove(&input) {
            return vec![MidiCommand::NoteOff { note }];
        }
        vec![]
    }

    pub fn panic(&mut self) -> Vec<MidiCommand> {
        let mut out: Vec<_> = self
            .active
            .values()
            .copied()
            .map(|note| MidiCommand::NoteOff { note })
            .collect();
        self.active.clear();
        out.push(MidiCommand::AllNotesOff);
        out
    }
}

pub fn profile_path(base: &Path, guid: &str) -> PathBuf {
    let safe: String = guid
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    base.join(format!("{safe}.json"))
}

pub fn save_profile(base: &Path, profile: &Profile) -> Result<(), Box<dyn std::error::Error>> {
    profile.validate()?;
    fs::create_dir_all(base)?;
    fs::write(
        profile_path(base, &profile.controller_guid),
        serde_json::to_vec_pretty(profile)?,
    )?;
    Ok(())
}

pub fn load_profile(base: &Path, guid: &str) -> Result<Profile, Box<dyn std::error::Error>> {
    let profile: Profile = serde_json::from_slice(&fs::read(profile_path(base, guid))?)?;
    profile.validate()?;
    Ok(profile)
}

pub fn note_name(note: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    format!("{}{}", NAMES[(note % 12) as usize], note as i16 / 12 - 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_cover_drum_rack() {
        let p = Profile::default_for("g", "n");
        assert_eq!(
            p.assignments.iter().map(|a| a.note).collect::<Vec<_>>(),
            (36..48).collect::<Vec<_>>()
        );
        assert!(p.validate().is_ok());
    }
    #[test]
    fn learn_rejects_duplicate_notes() {
        let mut p = Profile::default_for("g", "n");
        assert_eq!(
            p.learn(1, PhysicalInput::Button(20), 36),
            Err(ProfileError::DuplicateNote(36))
        );
    }
    #[test]
    fn learn_relocates_an_already_mapped_input() {
        let mut p = Profile::default_for("g", "n");
        let old_target = p.assignments[0].input.clone();
        let existing = p.assignments[5].input.clone();
        p.learn(0, existing.clone(), 36).unwrap();
        assert_eq!(p.assignments[0].input, existing);
        assert_eq!(p.assignments[5].input, old_target);
        assert!(p.validate().is_ok());
    }
    #[test]
    fn press_release_and_repeat() {
        let p = Profile::default_for("g", "n");
        let mut s = MidiState::default();
        assert_eq!(
            s.input(&p, PhysicalInput::DPadUp, true),
            vec![MidiCommand::NoteOn {
                note: 36,
                velocity: 100
            }]
        );
        assert!(s.input(&p, PhysicalInput::DPadUp, true).is_empty());
        assert_eq!(
            s.input(&p, PhysicalInput::DPadUp, false),
            vec![MidiCommand::NoteOff { note: 36 }]
        );
    }
    #[test]
    fn disconnect_cleans_all_notes() {
        let p = Profile::default_for("g", "n");
        let mut s = MidiState::default();
        s.input(&p, PhysicalInput::DPadUp, true);
        s.input(&p, PhysicalInput::DPadDown, true);
        assert_eq!(
            s.panic(),
            vec![
                MidiCommand::NoteOff { note: 36 },
                MidiCommand::NoteOff { note: 37 },
                MidiCommand::AllNotesOff
            ]
        );
    }
    #[test]
    fn profile_roundtrip() {
        let dir = std::env::temp_dir().join(format!("fsm-test-{}", std::process::id()));
        let p = Profile::default_for("guid", "stick");
        save_profile(&dir, &p).unwrap();
        assert_eq!(load_profile(&dir, "guid").unwrap(), p);
        let _ = fs::remove_dir_all(dir);
    }
}
