use std::sync::{Arc, Mutex};
use std::thread;

use cpal::{Sample, Stream};
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    FromSample, SizedSample,
};
use crate::oscillators::*;



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


pub fn live_thread_init<'a, T> (device: &cpal::Device, config: &cpal::StreamConfig, oscs: &mut Vec<Oscillator>) 
where
    T: SizedSample + FromSample<f32>,
{
    //let mut oscs = oscs;
    let _sample_rate = config.sample_rate.0 as f32;
    //let channels = config.channels as usize;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);


    let mut oscs = Arc::new(Mutex::new(&mut oscs));
    let channels = Arc::new(config.channels as usize);
    let device = Arc::new(device.clone());



    thread::spawn(move || {
        loop {
            let stream = device.build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, *channels, &mut oscs.lock().unwrap())
                },
                err_fn,
                None,
            ).unwrap();
            stream.play().unwrap(); 
        }
    });

    let _ = ();

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