//use cpal::traits::{DeviceTrait, HostTrait};

pub mod audiolib;
use audio_test::get_user_input;
use audiolib::*;
pub mod oscillators;
use oscillators::*;
use cpal::traits::{DeviceTrait, HostTrait};


fn main() {

    //let user_input = get_user_input();
    //let user_freq = user_input.trim().parse::<f32>().unwrap();
    let user_freq = 440f32;

    let mut osc1 = Oscillator::new_oscillator(Waveform::Square, 440_f32, 0.6f32);
    let mut osc2 = Oscillator::new_oscillator(Waveform::Sine, 620_f32, 0.6f32);
    let mut oscs = vec![&mut osc1, &mut osc2];

    live_thread_init::<f32>(oscs);

    //let _ = run::<f32>(&device, &config.into(), oscs);


}







