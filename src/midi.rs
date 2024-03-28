use midir::{Ignore, MidiInput, MidiInputConnection, MidiInputPort};
use std::{error::Error, str::Bytes, sync::{mpsc::Sender, Arc, Mutex, RwLock}, thread, vec};

use crate::syerr::{self, MidiError};


pub struct MidiModule {
    pub midi_input: MidiInput,
    pub midi_port: MidiInputPort
}

impl MidiModule {
    
    pub fn new (input_index: usize) -> Result<MidiModule, syerr::MidiError> {

        let mut midi_in = MidiInput::new("New Module Input")?;
        midi_in.ignore(Ignore::None);
        let in_ports = midi_in.ports();

        //let user_input = input_index.trim().parse::<usize>()?;

        let port = match in_ports.get(input_index).ok_or("input invalid") {
            Ok(p) => p,
            Err(_) => return Err(MidiError::new("Unresolved Input Index")),
        };

        return Ok(MidiModule { midi_input: midi_in, midi_port: port.clone() });
        
    }


    pub fn listen (self, tx: Arc<Mutex<Sender<MidiMessage>>>) -> Result<(), syerr::MidiError> {


        let port_name = &self.midi_input.port_name(&self.midi_port)?;
        println!(
            "Connection open, reading input from '{}'...",
            port_name
        );

        //let midi_msg: Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(vec![]));

            let _conn_in = self.midi_input.connect(
                &self.midi_port,
                "LRS-midi-listen",
                move |_stamp, message, data| {
                    /*
                    print!("\n{} (len = {}) : ", stamp, message.len());
                    for byte in message  {
                        print!("{:02x},", byte);
                    }
                    */
                    let tx = Arc::clone(&data);
                    let tx = tx.lock().unwrap();
                    let mut msg: Vec<u8> = vec![];
                    msg.extend_from_slice(message);
                    tx.send(midi_format(msg).unwrap()).unwrap();
                },
                tx.clone(),
            )?;

            loop {}   
    }

}

pub fn display_inputs() -> Result<Vec<usize>, syerr::MidiError> {
    let mut midi_in = MidiInput::new("reading input")?;
    midi_in.ignore(Ignore::None);

    let mut output_string = String::new();
    let mut indexes = vec![];

    let in_ports = midi_in.ports();
    match in_ports.len() {
        0 => return Err(MidiError::new("No Input Port Found !")),
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                output_string = format!("[{}]:{}\n", i, midi_in.port_name(p)?);
                indexes.push(i);
            }
        }
    };
    println!("{}", output_string);
    Ok(indexes)

}

pub enum MidiMessage {
    // Chan, Key, Vel
    NoteOn(u8, u8, u8),
    // Chan, Key, Vel
    NoteOff(u8, u8, u8),
    // Chan, Ctrl, Vel
    ControlChange(u8, u8, u8),
    // Chan, Prog number
    ProgramChange(u8, u8),
    // CHan, PB data
    PitchBend(u8, u8),
    // Chan, Key, Vel
    Aftertouch(u8, u8, u8),
    Sysex(Vec<u8>),
    MidiClock,
    MidiStart,
    MidiContinue,
    MidiStop,
    Reset,
}

impl MidiMessage {

    pub fn to_freq (message: MidiMessage) -> Result<f32, syerr::MidiError> {

        match message {
            MidiMessage::NoteOn(_, k ,_ ) | MidiMessage::Aftertouch(_, k, _) => {
                    //print!("{}", k);
                    if k <= 127 {
                        return Ok(440.0 * 2_f32.powf((k as f32 - 69.0)/12.0));
                    } else { return Err(MidiError::new("Unresolved Key")) }
                },
            _ => Ok(0.0)
        }
    }

    
}

pub fn midi_format (message: Vec<u8>) -> Result<MidiMessage, syerr::MidiError> {
    if message.len() == 0 {
        return Err(syerr::MidiError::new("Empty Message !"));
    }


    let byte = format!("{:02X}", message[0]);
    let mut fbyte_as_hex = byte.chars();
    let message_type = fbyte_as_hex.next().unwrap();
    let chan = fbyte_as_hex.next().unwrap() as u8;


    match message_type {
        '8' => { 
            return Ok(MidiMessage::NoteOff(chan, message[1], message[2]));
        },
        '9' => {
            return Ok(MidiMessage::NoteOn(chan, message[1], message[2]));
        },
        'A' => {
            return Ok(MidiMessage::Aftertouch(chan, message[1], message[2]));
        },
        'B' => {
            return Ok(MidiMessage::ControlChange(chan, message[1], message[2]));
        },
        'C' => {
            return  Ok(MidiMessage::ProgramChange(chan, message[1]));
        },
        'E' => {
            return Ok(MidiMessage::PitchBend(chan, message[1]));
        },
        _ => return Err(syerr::MidiError::new("Unresolved Midi Message type Byte"))
    }

    
}