use cpal::Sample;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use wavegen::{sine, wf};


fn main() {
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device available");
    let supported_config = device.default_output_config().unwrap();

    println!("Device: {},\nUsing config: {:?}\n", device.name().expect("flute"), supported_config);

    let config = supported_config.into();


    let stream = device.build_output_stream(&config, write_data, err_fn, None).unwrap();
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(3000));
}

fn write_data(data: &mut [f32], _: &cpal::OutputCallbackInfo) {
    /*
    let mut counter = 0;
    for sample in data.iter_mut() {
        let s = if (counter / 20) % 2 == 0 { &1.0 } else { &0.0 };
        counter = counter + 1;
        *sample = s.to_sample();
    }
    //println!("{:?}", data);
    */
    
    let waveform = wf!(f32, 44100., sine!(frequency: 100., amplitude: 10.));
    let some_samples: Vec<f32> = waveform.iter().take(44100).map(|s| s.to_sample()).collect();
    //println!("{:?}", some_samples);
    let mut counter = 0;
    for sample in data.iter_mut() {
        *sample = some_samples[counter];
        counter = counter + 1;
    }
    //println!("{:?}", data);


}

