use fighting_stick_core::MidiCommand;
use midir::{MidiOutput, MidiOutputConnection};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MidiError {
    #[error("não foi possível iniciar MIDI: {0}")]
    Init(String),
    #[error("porta MIDI não encontrada: {0}")]
    Port(String),
    #[error("não foi possível conectar: {0}")]
    Connect(String),
    #[error("erro ao enviar MIDI: {0}")]
    Send(String),
}

pub struct MidiOut {
    connection: Option<MidiOutputConnection>,
}
impl MidiOut {
    pub fn new() -> Self {
        Self { connection: None }
    }
    pub fn ports() -> Result<Vec<String>, MidiError> {
        let midi =
            MidiOutput::new("Fighting Stick MIDI").map_err(|e| MidiError::Init(e.to_string()))?;
        Ok(midi
            .ports()
            .iter()
            .filter_map(|p| midi.port_name(p).ok())
            .collect())
    }
    pub fn connect(&mut self, name: &str) -> Result<(), MidiError> {
        self.disconnect();
        let midi =
            MidiOutput::new("Fighting Stick MIDI").map_err(|e| MidiError::Init(e.to_string()))?;
        let port = midi
            .ports()
            .into_iter()
            .find(|p| midi.port_name(p).ok().as_deref() == Some(name))
            .ok_or_else(|| MidiError::Port(name.into()))?;
        self.connection = Some(
            midi.connect(&port, "Fighting Stick output")
                .map_err(|e| MidiError::Connect(e.to_string()))?,
        );
        Ok(())
    }
    pub fn disconnect(&mut self) {
        self.connection.take();
    }
    pub fn send(&mut self, command: MidiCommand) -> Result<(), MidiError> {
        let bytes: &[u8] = match command {
            MidiCommand::NoteOn { note, velocity } => &[0x90, note, velocity],
            MidiCommand::NoteOff { note } => &[0x80, note, 0],
            MidiCommand::AllNotesOff => &[0xB0, 123, 0],
        };
        if let Some(c) = &mut self.connection {
            c.send(bytes).map_err(|e| MidiError::Send(e.to_string()))?;
        }
        Ok(())
    }
}
impl Default for MidiOut {
    fn default() -> Self {
        Self::new()
    }
}
