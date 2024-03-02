use std::sync::{Arc, Mutex};
use std::thread;

use cpal::{Sample, Stream};
use cpal::{
    traits::{DeviceTrait, StreamTrait, HostTrait},
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


pub fn live_thread_init<'a, T> (oscs: Vec<&'a mut Oscillator>) 
where
    T: SizedSample + FromSample<f32>,
{
    // Audio Device Init
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available !!");
    let config = device.default_output_config().unwrap().config();
    println!("Device: {},\nUsing config: {:?}\n", device.name().expect("no name !!"), config);

    // Extract some variables from Host Config
    let _sample_rate = config.sample_rate.0 as f32;
    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);
    // Write Sample Rate in the oscs
    for osc in oscs {
        osc.sample_rate = Some(config.sample_rate.0 as f32);
    }

    // Prepare values for the Audio Thread
    let oscs = Arc::new(Mutex::new(oscs));
    let channels = Arc::new(config.channels);
    let device = Arc::new(device);


    // Start Audio Thread
    thread::spawn(move || {
        loop {
            let channels = Arc::clone(&channels);
            let device = Arc::clone(&device);
            let oscs = Arc::clone(&oscs);
            let stream = device.build_output_stream(
                &config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    write_data(data, *channels as usize, &mut oscs.lock().unwrap())
                },
                err_fn,
                None,
            ).unwrap();
            stream.play().unwrap(); 
        }
    });

    let _ = ();

}


pub fn write_data<'a, T>(output: &mut [T], channels: usize, oscs: &mut Vec<&mut Oscillator>)
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

pub fn osc_summing (inputs: &mut Vec<&mut Oscillator>) -> f32{
    let mut buffer = 0_f32;
    for o in inputs {
        buffer = buffer + o.tick();
    }
    buffer

}