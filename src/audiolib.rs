use std::sync::{Arc, Mutex};
use cpal::{Device, Stream, StreamConfig};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    FromSample, SizedSample,
};
use crate::dsp::oscillators::*;
use crate::dsp::modulation::*;
use crate::dsp::env::*;
use crate::midi;

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
//#[derive(Clone)]
pub enum Node {
    OscNode(Oscillator),
    ModNode(OscModulator),
    ProcessNode(ProcessNode),
    EnvNode(Env),
    MidiNode(midi::MidiModule)
}

#[derive(Clone)]
pub struct ProcessNode {
    pub input_node : Option<Arc<Mutex<Node>>>,
    host : Option<Arc<Mutex<HostConfig>>>
}

impl ProcessNode {

    pub fn new (source : Option<Arc<Mutex<Node>>>, host: Option<Arc<Mutex<HostConfig>>>) -> Self {
        return ProcessNode {
            input_node : source,
            host : host
        }
    }

    // Push audio context to process node
    pub fn context (&mut self, host: Arc<Mutex<HostConfig>>) {
        self.host = Some(host);
    }

    // The Make method is the closest to CPAL 
    // it runs a oscillator method in its core for now, but the match expression makes it generic
    pub fn make<'a, T> (&'a mut self) -> Stream
    where
        T: SizedSample + FromSample<f32>,
    {
        let host = self.host.as_mut().unwrap();
        let host = Arc::clone(&host);
        let host = host.lock().unwrap();
        //let host = host.as_mut().unwrap();

        // Extract some variables from Host Config
        let _sample_rate = host.config.sample_rate.0 as f32;
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let channels = host.config.channels as usize;

        //let mut input_node = self.input_node.clone();
        //let mut input_node = *input_node;
        
        let input_node = Arc::clone(&self.input_node.as_mut().unwrap());

        // This is where the magic happens !
        let stream = {
            host.device.build_output_stream(
            &host.config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
    
                for frame in data.chunks_mut(channels) {
                    let mut input_node = input_node.lock().unwrap();
                    let value: T = T::from_sample(
                        match &mut *input_node {
                                Node::OscNode(osc) => osc.process::<T>(),
                                Node::ModNode(oscmod) => oscmod.process::<T>(),
                                _ => 0.0  
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

// Route input 
impl Routable for ProcessNode {
    fn route (&mut self, input: Arc<Mutex<Node>>) {
        self.input_node = Some(input);
    }
}

// This trait allows the route_node method to be generic accros nodes types
pub trait Routable {
    fn route (&mut self, node: Arc<Mutex<Node>>);
}

pub trait Processable {
    fn process<T> (&mut self) -> f32
    where
        T: SizedSample + FromSample<f32>;
}