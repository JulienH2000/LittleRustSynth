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
/*
    fn calculate_square_output_from_freq(&self) -> f32 {
       self.amplitude * (((self.current_sample_index * self.frequency_hz * 2.0 * PI / self.sample_rate) + self.phase).sin()).signum()
    }
*/
    fn calculate_square_output_from_freq(&self) -> f32 {
        let mut value = 0.0;
        let phase = self.current_sample_index / (1.0/self.frequency_hz);
        let t = phase / 2.0*PI;
        if (phase < PI) {
            value = 1.0;
        } else {
            value = -1.0;
        }
        value = value + self.calc_poly_blep(t);
        value = value - self.calc_poly_blep((t + 0.5) % 1.0);
        value
       
  }
/*
    fn calculate_saw_output_from_freq(&mut self) -> f32 {
        self.next_sample_index();
        let mut output = 0.0;
        let mut k = 1f32;
        while !self.is_multiple_of_freq_above_nyquist(k) {
            let gain = -1f32.powf(k);
            output += gain * ((self.calculate_sine_output_from_freq(self.frequency_hz * k)) / k);
            k = k + 1.0;
            //println!("{}", self.frequency_hz * k);
        }
        self.amplitude * (0.5 - 1f32/PI * output) - 0.3
    }
*/
/*
    fn calculate_saw_output_from_freq(&mut self) -> f32 {
        self.next_sample_index();
        let period = 1.0 / self.frequency_hz;
        let phase = self.current_sample_index % period;
        let value = self.amplitude * ( (2.0 * phase / period) - 1.0);
        
        value
    }
*/
/*
    fn calculate_saw_output_from_freq(&mut self) -> f32 {
        let mut amp =self.amplitude;
        let mut output = 0.0;
        let period = 1.0 / self.frequency_hz;
        let mut k = 1.0;
        let mut dd;
        let phase = self.current_sample_index / period;
        let mut hphase = phase;
        while !self.is_multiple_of_freq_above_nyquist(k)  {
            dd = (phase * k * 2.0 * PI).cos();
            output = output + (amp * dd * (hphase * 2.0 * PI).sin());
            k = k + 1.0;
            hphase = phase * k;
            amp = 1.0 / k;
        }
        output
    }
*/
    fn calculate_saw_output_from_freq(&mut self) -> f32 {
        let phase = self.current_sample_index / (1.0/self.frequency_hz);
        let mut value = (2.0 * phase / 2.0*PI) - 1.0;
        value = value - self.calc_poly_blep(phase / 2.0*PI);
        value

    }
    


    fn calculate_triangle_output_from_freq(&self) -> f32 {
        todo!()
    }

    fn is_multiple_of_freq_above_nyquist(&self, multiple: f32) -> bool {
        self.frequency_hz * multiple > self.sample_rate / 2.0
        
    }

    fn calc_poly_blep(&self, t: f32) -> f32 {
        let mut t = t;
        let dt = self.current_sample_index / 2.0 * PI;

        if t < dt {
            t = t / dt;
            return t + t - t * t - 1.0;
        } else if t > 1.0 - dt {
            t = t - 1.0 / dt;
            return t * t + t + 1.0;
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