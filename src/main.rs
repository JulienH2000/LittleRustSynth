//use cpal::traits::{DeviceTrait, HostTrait};

pub mod audiolib;
use audio_test::get_user_input;
use audiolib::*;
pub mod oscillators;
use oscillators::*;
use cpal::traits::{DeviceTrait, HostTrait};


fn main() {

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available !!");
    let config = device.default_output_config().unwrap().config();
    println!("Device: {},\nUsing config: {:?}\n", device.name().expect("no name !!"), config);

    //let user_input = get_user_input();
    //let user_freq = user_input.trim().parse::<f32>().unwrap();
    let user_freq = 440f32;

    let osc1 = Oscillator::new_oscillator(Waveform::Square, &config, 440_f32, 0.6f32);
    let osc2 = Oscillator::new_oscillator(Waveform::Sine, &config, 620_f32, 0.6f32);
    let oscs = vec![osc1, osc2];

    let _ = run::<f32>(&device, &config.into(), oscs);

}






