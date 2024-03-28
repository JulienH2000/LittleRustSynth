use std::error::Error;
use std::fmt;

use midir::{ConnectError, InitError, PortInfoError};


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

impl From<std::io::Error> for SynthError {
    fn from(error: std::io::Error) -> Self {
        SynthError {
            message: format!("An synthesis error occurred: {}", error),
        }
    }
}

impl fmt::Display for SynthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

// Midi Error

#[derive(Debug, Clone)]
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

impl From<std::io::Error> for MidiError {
    fn from(error: std::io::Error) -> Self {
        MidiError {
            message: format!("An Midi error occurred: {}", error),
        }
    }
}
impl From<InitError> for MidiError {
    fn from(error: InitError) -> Self {
        MidiError {
            message: format!("An Midi error occurred: {}", error),
        }
    }
}
impl From<PortInfoError> for MidiError {
    fn from(error: PortInfoError) -> Self {
        MidiError {
            message: format!("An Midi error occurred: {}", error),
        }
    }
}
impl From<ConnectError<midir::MidiInput>> for MidiError {
    fn from(error: ConnectError<midir::MidiInput>) -> Self {
        MidiError {
            message: format!("An Midi error occurred: {}", error),
        }
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

impl From<std::io::Error> for AudioError {
    fn from(error: std::io::Error) -> Self {
        AudioError {
            message: format!("An audio engine error occurred: {}", error),
        }
    }
}