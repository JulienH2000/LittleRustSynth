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


    // Init Host
    let mon_host = HostConfig::new();
    let mon_sample_rate = mon_host.config.sample_rate;

    let mon_oscillateur = SourceNode::OscNode(OscNode::make_from(osc1, osc2, mon_sample_rate, rx));
    let mut mon_stream = RenderNode::new(mon_oscillateur);
    let ma_sortie = mon_stream.make::<f32>(mon_host);

    loop {
        ma_sortie.play().unwrap();
        let new_freq = get_user_input().trim().parse().unwrap_or(440.0);
        tx.send(new_freq).unwrap();
    }
}







