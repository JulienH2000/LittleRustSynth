use cpal::Sample;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    FromSample, SizedSample,
};
use core::f32::consts::PI;
use std::process::Output;
//use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

#[derive(Clone)]
pub struct Oscillator {
    pub sample_rate: f32,
    pub waveform: Waveform,
    pub current_sample_index: f32,
    pub frequency_hz: f32,
    pub amplitude: f32,
    pub phase: f32
}

impl Oscillator {
    pub fn new_oscillator (wave: Waveform, config: &cpal::StreamConfig, freq: f32, amp: f32) -> Oscillator {
        return Oscillator {
            sample_rate: config.sample_rate.0 as f32,
            waveform: wave,
            current_sample_index: 0f32,
            frequency_hz: freq,
            amplitude: amp,
            phase: 1f32
        }
    }

    fn next_sample_index (&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
    }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        self.amplitude * ((self.current_sample_index * freq * 2.0 * PI / self.sample_rate) + self.phase).sin()
    }
    fn calculate_square_output_from_freq(&self) -> f32 {
       self.amplitude * (((self.current_sample_index * self.frequency_hz * 2.0 * PI / self.sample_rate) + self.phase).sin()).signum()
    }
    fn calculate_saw_output_from_freq(&mut self) -> f32 {
        self.next_sample_index();
        let mut output = 0.0;
        let mut k = 1f32;
        while !self.is_multiple_of_freq_above_nyquist(k) {
            let gain = -1f32.powf(k);
            output += gain * ((self.calculate_sine_output_from_freq(self.frequency_hz * k)) / k);
            k = k + 1.0;
        }
        self.amplitude * (0.5 - 1f32/PI * output)
    }
    fn calculate_triangle_output_from_freq(&self) -> f32 {
        todo!()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency_hz * multiple > self.sample_rate / 2.0
        
    }

    fn calculate_from_sine(&mut self, harmonic_index_increment: i32, gain_exponent: f32){
        
    }

    fn sine_wave(&mut self) -> f32 {
        self.next_sample_index();
        self.calculate_sine_output_from_freq(self.frequency_hz)
    }
    fn square_wave(&mut self) -> f32 {
        self.next_sample_index();
        self.calculate_square_output_from_freq()
    }
    fn saw_wave(&mut self) -> f32 {
        //self.next_sample_index();
        self.calculate_saw_output_from_freq()
    }
    fn triangle_wave(&mut self) -> f32 {
        self.next_sample_index();
        self.calculate_triangle_output_from_freq()
    }

    fn tick(&mut self) -> f32 {
        
        match self.waveform {
            Waveform::Sine => self.sine_wave(),
            Waveform::Square => self.square_wave(),
            Waveform::Saw => self.saw_wave(),
            Waveform::Triangle => self.triangle_wave(),
        }
    }
}
/*
fn sine (osc: &mut Oscillator, next_value: &mut dyn FnMut() -> f32) {
    let mut clock = osc.current_sample_index;
    let mut next_value = move || {
        clock = (clock + 1.0) % osc.sample_rate;
        (clock * osc.frequency_hz * 2.0 * PI / osc.sample_rate).sin()
    };
} */

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, osc: Oscillator, dur: u64) -> Result<(), &'static str>
where
    T: SizedSample + FromSample<f32>,
{
    let mut osc = osc.clone();

    let _sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let time_at_start = std::time::Instant::now();
    println!("Time at start: {:?}", time_at_start);

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    //let osc_mutex = Arc::new(Mutex::new(osc));
    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut osc)
        },
        err_fn,
        None,
    ).unwrap();
    stream.play().unwrap();

    std::thread::sleep(std::time::Duration::from_millis(dur));

    Ok(())
}


pub fn write_data<T>(output: &mut [T], channels: usize, osc: &mut Oscillator)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(osc.tick());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}