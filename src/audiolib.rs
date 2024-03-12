use core::fmt;
use std::fmt::write;
use std::sync::{Arc, Mutex};
use cpal::{Device, Stream, StreamConfig};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    FromSample, SizedSample,
};
use crate::dsp::oscillators::*;
use crate::dsp::modulation::*;
use std::any::Any;

pub struct HostConfig {
    pub device: Device,
    pub config: StreamConfig
}

impl HostConfig {
    pub fn new () -> Self {
        // Audio Device Init
        let host = cpal::default_host();
        let device = host.default_output_device().expect("no output device available !!");
        let config = device.default_output_config().unwrap().config();
        println!("Device: {},\nUsing config: {:?}\n", device.name().expect("no name !!"), config);

        // New Host Instance
        HostConfig {
            device: device,
            config: config
        }
    }
}

// Node type Enum
#[derive(Clone)]
pub enum Node {
    OscNode(Oscillator),
    ModNode(OscModulator),
    ProcessNode,
}

// Impl display for the "see" method
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OscNode(_osc) => write!(f, "OscNode"),
            Self::ModNode(_mod) => write!(f, "ModNode"),
            Self::ProcessNode => write!(f, "ProcessNode"),
        }
    }
}

/*
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        match (*self, *other) {
            (Node::ProcessNode, Node::ProcessNode) => Some(Ordering::Equal),
            (Node::ProcessNode, _) => Some(Ordering::Less),
            (_, Node::ProcessNode) => Some(Ordering::Greater),
            (Node::ModNode(_), Node::ModNode(_)) => Some(Ordering::Equal),
            (Node::ModNode(_), Node::OscNode(_)) => Some(Ordering::Less),
            (Node::OscNode(_), Node::ModNode(_)) => Some(Ordering::Greater),
            (Node::OscNode(_), Node::OscNode(_)) => Some(Ordering::Equal),
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::ProcessNode, Node::ProcessNode) => true,
            (Node::OscNode(_), Node::OscNode(_)) => true,
            (Node::ModNode(_), Node::ModNode(_)) => true,
            _ => false,
        }
    }
}

impl Eq for Node {}
*/

impl Node {
    pub fn as_any(&self) -> &dyn Any {
        self
    }
}


pub struct ProcessNode {
    input_node : Option<Node>,
    host : Arc<Mutex<Option<HostConfig>>>
}

impl ProcessNode {

    pub fn new (source : Option<Node>, host: Arc<Mutex<Option<HostConfig>>>) -> Self {
        return ProcessNode {
            input_node : source,
            host : host
        }
    }

    // The Make method is the closest to CPAL 
    // it runs a oscillator method in its core for now, but the match expression makes it generic
    pub fn make<'a, T> (&'a mut self) -> Stream
    where
        T: SizedSample + FromSample<f32>,
    {
        let host = Arc::clone(&self.host);
        let mut host = host.lock().unwrap();
        let host = host.as_mut().unwrap();

        // Extract some variables from Host Config
        let _sample_rate = host.config.sample_rate.0 as f32;
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let channels = host.config.channels as usize;

        let mut input_node = self.input_node.clone();

        // This is where the magic happens !
        let stream = {
            host.device.build_output_stream(
            &host.config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let value: T = T::from_sample(
                        match &mut input_node {
                            Some(node) => match node {
                                Node::OscNode(osc) => osc.process::<T>(),
                                Node::ModNode(oscmod) => oscmod.process::<T>(),
                                _ => 0.0  
                            },
                            None => 0.0
                        }
                    );
                    for sample in frame.iter_mut() {
                        *sample = value;
                    }
                }
                
            },
            err_fn,
            None,
        ).unwrap()
    };
    stream
    }
}

impl Routable for ProcessNode {
    fn route (&mut self, input: Node) {
        self.input_node = Some(input);
    }
}


pub trait Routable {
    fn route (&mut self, node: Node);
}