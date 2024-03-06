use std::sync::{Arc, Mutex};
use cpal::{Device, Stream, StreamConfig};
use cpal::{
    traits::{DeviceTrait, HostTrait},
    FromSample, SizedSample,
};
use crate::dsp::oscillators::*;

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
            device,
            config
        }
    }
}

#[derive(Clone)]
pub enum SourceNode {
    OscNode(Oscillator),
    AudioNode
}

pub struct ProcessNode {
    input_node : SourceNode,
    host : Arc<Mutex<Option<HostConfig>>>
}

impl ProcessNode {

    pub fn new (source : SourceNode, host: HostConfig) -> Self {
        return ProcessNode {
            input_node : source,
            host : Arc::new(Mutex::new(Some(host)))
        }
    }

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
        let stream = {
            host.device.build_output_stream(
            &host.config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                for frame in data.chunks_mut(channels) {
                    let value: T = T::from_sample(
                        match &mut input_node {
                            SourceNode::OscNode(osc) => osc.process::<T>(),
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

        //thread sleep is used to hold artificially stream.play() in scope
        //std::thread::sleep(std::time::Duration::from_millis(2000));   

    stream
    }
}