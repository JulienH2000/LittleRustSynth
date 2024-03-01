use cpal::Sample;
use cpal::{
    traits::{DeviceTrait, StreamTrait},
    FromSample, SizedSample,
};
use core::f32::consts::PI;
use std::process::Output;
use std::sync::mpsc::channel;
use std::thread;
use std::sync::{Arc, Mutex, MutexGuard};

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
    pub phase_shift: f32,
}

impl Oscillator {
    pub fn new_oscillator (wave: Waveform, config: &cpal::StreamConfig, freq: f32, amp: f32) -> Oscillator {
        return Oscillator {
            sample_rate: config.sample_rate.0 as f32,
            waveform: wave,
            current_sample_index: 0f32,
            frequency_hz: freq,
            amplitude: amp,
            phase_shift: 1f32,
        }
    }

    fn next_sample_index (&mut self) {
        self.current_sample_index = (self.current_sample_index + 1.0) % self.sample_rate;
    }

    fn calculate_sine_output_from_freq(&self, freq: f32) -> f32 {
        self.amplitude * ((self.current_sample_index * freq * 2.0 * PI / self.sample_rate) + self.phase_shift).sin()
    }

    fn calculate_square_output_from_freq(&self) -> f32 {
        let mut output = 0.0;
        let freq = self.frequency_hz;
        let phase = self.current_sample_index * freq * 2.0 * PI / self.sample_rate;
        let period = self.sample_rate / freq;
        let t = phase / period;
        let half_phase = self.sample_rate / 2.0;

        // Naive Square gen

        output = self.amplitude * ((phase).sin()).signum();

        // PolyBLEP Substraction 

        //output = output + self.calc_poly_blep(t);
        //output = output - self.calc_poly_blep((t + 0.5) % 1.0);
        output
       
  }

    fn calculate_saw_output_from_freq(&mut self) -> f32 {
        let freq = self.frequency_hz;
        let index = self.current_sample_index;
        let phase = self.current_sample_index * freq * 2.0 * PI / self.sample_rate;
        let period = self.sample_rate / freq;
        let t = phase / period;

        // Naive sawtooth gen
        let mut output = (2.0 * (index % period) / period) - 1.0;

        //let mut output = (2.0 * t) - 1.0;
        output = output * self.amplitude;

        // PolyBLEP Substraction

        output = output - self.calc_poly_blep(t);
        output
    }

/*
Legacy Band-limited Gen 
        let mut output = 0.0;
        let mut k = 1f32;
        while !self.is_multiple_of_freq_above_nyquist(k) {
            let gain = -1f32.powf(k);
            output += gain * ((self.calculate_sine_output_from_freq(self.frequency_hz * k)) / k);
            k = k + 1.0;
            //println!("{}", self.frequency_hz * k);
        }
        output = self.amplitude * (0.5 - 1f32/PI * output) - 0.3;
*/

    


    fn calculate_triangle_output_from_freq(&self) -> f32 {
        todo!()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency_hz * multiple > self.sample_rate / 2.0
        
    }

    fn calc_poly_blep(&self, t: f32) -> f32 {
        /* t = phase / 2 PI */
        let mut t = t;
        //let dt = self.current_sample_index / 2.0 * PI;
        let dt = self.current_sample_index / 2.0 * PI;

        if t < dt {
            t = t / dt;
            return t + t - t * t - 1.0;
        } else if t > 1.0 - dt {
            t = t - 1.0 / dt;
            return t * t + t + t + 1.0;
        } else {
            return 0.0;
        }
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
        self.next_sample_index();
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


pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig, oscs: Vec<Oscillator>, dur: u64) -> Result<(), &'static str>
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

    std::thread::sleep(std::time::Duration::from_millis(dur));

    Ok(())
}


pub fn write_data<'a, T>(output: &mut [T], channels: usize, oscs: &mut Vec<Oscillator>)
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

/* multithreading sum
pub fn osc_buffering (osc1: & mut Oscillator, osc2: & mut Oscillator) -> f32{
        let mut osc1 = osc1.clone();
        let mut osc2 = osc2.clone();
        let mut osc1 = Arc::new(Mutex::new(&mut osc1));
        let mut osc2 = Arc::new(Mutex::new(&mut osc2));
        let buffer1 = thread::spawn(move || {
            osc1.lock().unwrap().tick()
        });
        let buffer2 = thread::spawn(move || {
            osc2.lock().unwrap().tick()
        });

        let output = buffer1.join().unwrap() + buffer2.join().unwrap();
        output

    }
*/

//single thread sum

pub fn osc_summing (inputs: &mut Vec<Oscillator>) -> f32{
    let mut buffer = 0_f32;
    for o in inputs {
        buffer = buffer + o.tick();
    }
    buffer

}