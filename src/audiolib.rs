use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::{cell::RefCell, sync::{mpsc::channel, Arc, Mutex, RwLock}};
use cpal::{Device, Sample, SampleRate, Stream, StreamConfig};
use cpal::{
    traits::{DeviceTrait, StreamTrait, HostTrait},
    FromSample, SizedSample,
};
use crate::oscillators::*;

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
    OscNode(OscNode),
    AudioNode
}

pub struct RenderNode {
    input_node : SourceNode,
    host : Arc<Mutex<Option<HostConfig>>>
}

impl RenderNode {

    pub fn new (source : SourceNode, host: HostConfig) -> Self {
        return RenderNode {
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
                            SourceNode::OscNode(osc) => osc.process::<T>(channels),
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

    /*
    // Buffers from OscNode to output
    pub fn run<'a, T>(&'a mut self, output: &'a mut [T], channels: usize)
    where
        T: Sample + FromSample<f32>,
    {
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(
                match &mut self.input_node {
                    SourceNode::OscNode(osc) => osc.process::<T>(channels),
                    _ => 0.0  
                }
            );
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
    }
    */
}

#[derive(Clone)]
pub struct OscNode {
    osc1 : Oscillator,
    osc2 : Oscillator,
    pub current_sample_rate: f32,
    single_mode_flag : bool,
    inbox : Arc::<Mutex<Option<Receiver<f32>>>>
}

impl OscNode {


    pub fn make_from (osc1: Oscillator, osc2: Oscillator, sample_rate: SampleRate, inbox: Receiver<f32>) -> Self {
        OscNode {
            osc1 : osc1,
            osc2 : osc2,
            current_sample_rate: sample_rate.0 as f32,
            single_mode_flag : true,
            inbox : Arc::new(Mutex::new(Some(inbox)))
        }
    }

    // Ocsillator to buffer 
    pub fn process<'a, T>(&'a mut self, channels: usize) -> f32
    where
        T: Sample + FromSample<f32>,
    {
        // ça c'est de la method de fumeur de mauvais shit 
        let inbox = Arc::clone(&self.inbox);
        let mut inbox = inbox.lock().unwrap();
        match inbox.as_mut().unwrap().try_recv() {
            Ok(msg) => self.check_inbox(msg),
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {panic!("inbox Disconnected !!")},
        }
        /*
        for frame in output.chunks_mut(channels) {
            let value: T = T::from_sample(
                self.tick()
            );
            for sample in frame.iter_mut() {
                *sample = value;
            }
        }
        */
        return self.tick();
    }

    pub fn tick(&mut self) -> f32 {
        let current_sample_rate = self.current_sample_rate; 
        let tick_waveform = |osc: &mut Oscillator| 
            match osc.waveform {
                Waveform::Sine => osc.sine_wave(&current_sample_rate),
                Waveform::Square => osc.square_wave(&current_sample_rate),
                Waveform::Saw => osc.saw_wave(&current_sample_rate),
                Waveform::Triangle => osc.triangle_wave(&current_sample_rate),
        };
        
        let osc1_sample = tick_waveform(&mut self.osc1);
        let osc2_sample = tick_waveform(&mut self.osc2);

        if self.single_mode_flag == true {
            return osc1_sample
        } else {
            return osc1_sample * osc2_sample;
        }

    }

    fn check_inbox (&mut self, msg: f32) {
        self.osc1.frequency_hz = msg;
    }

}