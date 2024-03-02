use std::sync::{Arc, Mutex};
use std::thread;

use cpal::{Device, Sample, Stream, StreamConfig};
use cpal::{
    traits::{DeviceTrait, StreamTrait, HostTrait},
    FromSample, SizedSample,
};
use crate::oscillators::*;


pub struct StreamOutput;


// RUN REQUIERE MAIN THREAD SLEEP
/*
pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, oscs: Vec<Oscillator>) -> Result<(), &'static str>
where
    T: SizedSample + FromSample<f32>,
{
    let mut oscs = oscs.clone();

    let _sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);


    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut oscs)
        },
        err_fn,
        None,
    ).unwrap();
    stream.play().unwrap();

    //std::thread::sleep(std::time::Duration::from_millis(2000));

    Ok(())
}

*/


impl StreamOutput {
    pub fn run<'a, T> (oscs: &Vec<Oscillator>) -> Stream
    where
        T: SizedSample + FromSample<f32>,
    {
        // Init Host
        let host = HostConfig::new();

        //let mut oscs = oscs.clone();


        // Extract some variables from Host Config
        let _sample_rate = host.config.sample_rate.0 as f32;
        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
        let channels = host.config.channels as usize;
        // Write Sample Rate in the oscs
        let mut buf:Vec<Oscillator> = vec![];
        for osc in oscs {
            buf.push( Oscillator{ sample_rate : Some(host.config.sample_rate.0 as f32), ..osc.clone() });
        }
        let mut oscs = buf;

        let stream = host.device.build_output_stream(
            &host.config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut oscs)
            },
            err_fn,
            None,
        ).unwrap();
        //thread is used to hold artificially stream.play() in scope
        //std::thread::sleep(std::time::Duration::from_millis(2000));

        stream

    }
}


pub fn write_data<'a, T>(output: &mut [T], channels: usize, oscs: &mut Vec<Oscillator>)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(osc_summing(oscs));
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}


//single thread sum

pub fn osc_summing (inputs: &mut Vec<Oscillator>) -> f32{
    let mut buffer = 0_f32;
    for o in inputs {
        buffer = buffer + o.tick();
    }
    buffer

}

pub struct HostConfig {
    device: Device,
    config: StreamConfig
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