use crate::dsp::oscillators::*;
use crate::audiolib::*;
use crate::toolbox::*;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, TryRecvError};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    FromSample, SizedSample,
};


#[derive(Clone)]
pub struct OscModulator {
    pub label: String,
    pub oscs: Vec<Oscillator>,
    pub inbox : Arc::<Mutex<Option<Receiver<String>>>>,
    pub mod_index : f32
}

impl OscModulator {

    pub fn new (label: String, inbox: Option<Receiver<String>>, mod_index: f32) -> OscModulator {

        return OscModulator {
            label : label,
            oscs : vec![],
            inbox : Arc::new(Mutex::new(inbox)),
            mod_index : mod_index
        }
    }

    pub fn process<T> (&mut self) -> f32 
    where
        T: SizedSample + FromSample<f32>,
    {

        if self.oscs.len() == 0 {
            panic!("Node oscillators!");
        }
        
        let inbox = Arc::clone(&self.inbox);
        let mut inbox = inbox.lock().unwrap();
        match inbox.as_mut() {
            Some(inbox) => match inbox.try_recv() {
                Ok(msg) => self.check_inbox(msg),
                Err(TryRecvError::Empty) => {},
                Err(TryRecvError::Disconnected) => {panic!("inbox Disconnected !!")},
            },
            None => {}
        }

        let mut buffer: f32 = 0.0;

        for osc in &mut self.oscs {
            buffer *= self.mod_index;
            buffer += osc.process::<T>();
        }
        return buffer;
    }

    fn check_inbox (&mut self, msg: String) {
        /*
        Message syntaxe :
        parameter-value
        */
        let command: Vec<&str> = msg.trim().split("&&").collect();

        for arg in command {
            let args: Vec<&str> = arg.trim().split('-').collect();
            match args[0] {
                "modindex" => self.mod_index = args[1].parse::<f32>().unwrap(),
                _ => ()
            }
        }
    }


}

impl Routable for OscModulator {
    fn route (&mut self, node: Node) {
        let osc = match node {
            Node::OscNode(osc) => osc,
            _ => panic!()
        };
        self.oscs.push(osc);
    }
}