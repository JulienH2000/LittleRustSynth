//use cpal::traits::{DeviceTrait, HostTrait};

pub mod audiolib;
use audio_test::get_user_input;
use audiolib::*;
pub mod oscillators;
use oscillators::*;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{cell::RefCell, sync::{mpsc::channel, Arc, Mutex, RwLock}};


fn main() {

    //let user_input = get_user_input();
    //let user_freq = user_input.trim().parse::<f32>().unwrap();
    let user_freq = 440f32;

    let mut osc1 = Oscillator::new_oscillator(Waveform::Sine, 440_f32, 0.6f32);
    let mut osc2 = Oscillator::new_oscillator(Waveform::Sine, 620_f32, 0.6f32);
    //let mut oscs = vec![osc1];

    //live_thread_init::<f32>(oscs);

    let (tx, rx) = channel();

    //let _ = run::<f32>(&device, &config.into(), oscs);
    let stream = StreamOutput::make::<f32>(osc1, osc2, rx);

    

    loop {
        stream.play().unwrap();
        let new_freq = 2000.0;
        let new_freq = get_user_input().trim().parse().unwrap();
        tx.send(new_freq).unwrap();
        

    }
}







