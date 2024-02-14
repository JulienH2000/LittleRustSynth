use cpal::traits::{DeviceTrait, HostTrait};

mod audiolib;
use audiolib::run;

fn main() {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let config = device.default_output_config().unwrap();

    println!("Device: {},\nUsing config: {:?}\n", device.name().expect("flute"), config);

    let _ = run::<f32>(&device, &config.into(), 440.0, 1000);

}




