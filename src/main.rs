//use cpal::traits::{DeviceTrait, HostTrait};

mod audiolib;
use audio_test::get_user_input;
use audiolib::*;
use cpal::traits::{DeviceTrait, HostTrait};


fn main() {

    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available !!");
    let config = device.default_output_config().unwrap().config();
    println!("Device: {},\nUsing config: {:?}\n", device.name().expect("no name !!"), config);

    //let user_input = get_user_input();
    //let user_freq = user_input.trim().parse::<f32>().unwrap();
    let user_freq = 220f32;

    let osc = Oscillator::new_oscillator(Waveform::Saw, &config, user_freq, 0.8f32);

    let _ = run::<f32>(&device, &config.into(), osc, 2000);

}





