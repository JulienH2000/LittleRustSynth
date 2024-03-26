use std::error::Error;
use std::fmt;


//########################
   // SYnthERRors
//########################


// Synth Error

#[derive(Debug)]
pub struct SynthError {
    message: String,
}

impl SynthError {
    pub fn new(message: &str) -> SynthError {
        SynthError {
            message: message.to_string(),
        }
    }
}

impl Error for SynthError {}

impl fmt::Display for SynthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Midi Error

#[derive(Debug)]
pub struct MidiError {
    message: String,
}

impl MidiError {
    pub fn new(message: &str) -> MidiError {
        MidiError {
            message: message.to_string(),
        }
    }
}

impl Error for MidiError {}

impl fmt::Display for MidiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Audio Engine Error

#[derive(Debug)]
pub struct AudioError {
    message: String,
}

impl AudioError {
    pub fn new(message: &str) -> AudioError {
        AudioError {
            message: message.to_string(),
        }
    }
}

impl Error for AudioError {}

impl fmt::Display for AudioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}