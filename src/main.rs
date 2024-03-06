//use cpal::traits::{DeviceTrait, HostTrait};

pub mod audiolib;
use audio_test::get_user_input;
use audiolib::*;
use cpal::traits::StreamTrait;
use std::sync::mpsc::channel;
pub mod dsp;
use crate::dsp::oscillators::*;


fn main() {

    //let user_input = get_user_input();
    //let user_freq = user_input.trim().parse::<f32>().unwrap();
    let _user_freq = 440f32;

    //let osc1 = Oscillator::new(Waveform::Square, 440_f32, 0.6f32);
    //let osc2 = Oscillator::new(Waveform::Square, 620_f32, 0.6f32);
    //let mut oscs = vec![osc1];

    //live_thread_init::<f32>(oscs);

    let (tx, rx) = channel();


    // Init Host
    let mon_host = HostConfig::new();
    let mon_sample_rate = mon_host.config.sample_rate;

    //let mon_oscillateur = SourceNode::OscNode(OscNode::make_from(osc1, osc2, mon_sample_rate, rx));
    let mon_oscillateur = SourceNode::OscNode(Oscillator::new(Waveform::Triangle, mon_sample_rate, rx, 440_f32, 1f32));
    let mut mon_stream = ProcessNode::new(mon_oscillateur, mon_host);
    let ma_sortie = mon_stream.make::<f32>();

    loop {
        ma_sortie.play().unwrap();
        let message = get_user_input();
        tx.send(message).unwrap();
    }
}







