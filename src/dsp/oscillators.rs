use core::f32::consts::PI;
use std::sync::{Arc, Mutex};
use cpal::{Sample, SampleRate};
use cpal::FromSample;
use crate::audiolib::*;


#[derive(Copy, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

#[derive(Clone)]
pub struct Oscillator {
    pub label: String,
    pub waveform: Waveform,
    pub current_sample_index: f32,
    pub last_sample: f32,
    pub frequency_hz: f32,
    pub amplitude: f32,
    pub phase_shift: f32,
    pub current_sample_rate: f32,
    pub phase : f32,
    pub phase_incr : f32
}

impl Oscillator {
    pub fn new (label: String,
                wave: Waveform, 
                sample_rate: Option<SampleRate>, 
                freq: f32, 
                amp: f32
                ) -> Oscillator {
        let sample_rate = match sample_rate {
            Some(sr) => sr.0 as f32 ,
            None => 48000_f32
        };
        return Oscillator {
            label: label,
            waveform: wave,
            current_sample_index: 0f32,
            last_sample: 0f32,
            frequency_hz: freq,
            amplitude: amp,
            phase_shift: 1f32,
            current_sample_rate: sample_rate,
            phase : 0.0,
            phase_incr: 0.0
        }
    }

    pub fn new_empty () -> Oscillator {
        return Oscillator {
            label: "Default_label".to_string(),
            waveform: Waveform::Sine,
            current_sample_index: 0f32,
            last_sample: 0f32,
            frequency_hz: 440.0,
            amplitude: 0.0,
            phase_shift: 1f32,
            current_sample_rate: 44100.0,
            phase : 0.0,
            phase_incr: 0.0
        }
    }


    // Ocsillator processing
    pub fn process<'a, T>(&'a mut self) -> f32
    where
        T: Sample + FromSample<f32>,
    {
        self.update_increment();
        return self.next_sample();
    }

    // Push audio context to oscillator
    pub fn context (&self, host: Arc<Mutex<HostConfig>>) -> Self {

        let host = Arc::clone(&host);
        let host = host.lock().unwrap();
        //let host = host.as_mut().unwrap();

        return Oscillator {
            current_sample_rate : host.config.sample_rate.0 as f32,
            label : self.label.clone(),
            ..*self
        }
    }

    pub fn next_sample(&mut self) -> f32 {
        let output = match self.waveform {
            Waveform::Sine => self.sine_wave(),
            Waveform::Square => self.square_wave(),
            Waveform::Saw => self.saw_wave(),
            Waveform::Triangle => self.triangle_wave(),
        };

        self.phase += self.phase_incr;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        return output;
    }

    fn next_index (&mut self) {
        let sample_rate = self.current_sample_rate;
        self.current_sample_index = (self.current_sample_index + 1.0) % sample_rate;
    }

    fn update_increment(&mut self) {
        let phase_incr = self.frequency_hz * 2.0 * PI / self.current_sample_rate;
        self.phase_incr = phase_incr;
    }

    fn calc_poly_blep(&self, t: f32) -> f32 {
        let mut t = t;
        let dt = self.phase_incr / 2.0 * PI;

        if t < dt {
            t = t / dt;
            return t+t - t*t - 1.0;
        } else if t > 1.0 - dt {
            t = (t - 1.0) / dt;
            return t*t + t+t + 1.0;
        } else {
            return 0.0;
        }
    }

    pub fn sine_wave(&mut self) -> f32 {
        self.next_index();
        let output = self.phase.sin();
        output
    }
    pub fn square_wave(&mut self) -> f32 {
        let freq = self.frequency_hz;
        let t = self.phase / freq;

        self.next_index();

        // Naive Square gen
        
        let mut output;
        if self.phase < PI {
            output = 1.0;
        } else {
            output = -1.0;
        }
        

        // PolyBLEP Substraction 
        output = output + self.calc_poly_blep(t);
        output = output - self.calc_poly_blep((t + 0.5) % 1.0);
        output
    }
    pub fn saw_wave(&mut self) -> f32 {
        let freq = self.frequency_hz;
        let phase = self.phase;
        let period = self.current_sample_rate / freq;
        let t = phase / period;

        self.next_index();
        
        // Naive sawtooth gen
        let mut output = (2.0 * phase / 2.0 * PI) - 1.0;

        output = output * self.amplitude;

        // PolyBLEP Substraction
        output = output - self.calc_poly_blep(t);
        output
    }
    pub fn triangle_wave(&mut self) -> f32 {
        let freq = self.frequency_hz;
        let period = self.current_sample_rate / freq;
        let phase = self.phase;
        let t = phase / period;

        self.next_index();
        // Naive Square gen
        let mut output;
        if self.phase < PI {
            output = 1.0;
        } else {
            output = -1.0;
        }
        output = output * self.amplitude;

        // PolyBLEP Substraction
        output = output + self.calc_poly_blep(t);
        output = output - self.calc_poly_blep((t + 0.5) % 1.0);

        //Leaky Integration
            // Leaky integrator: y[n] = A * x[n] + (1 - A) * y[n-1]
        output = self.phase_incr * output + (1.0 - self.phase_incr) * self.last_sample;
        self.last_sample = output;

        output
    }
}
