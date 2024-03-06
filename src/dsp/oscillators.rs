use core::f32::consts::PI;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::sync::{Arc, Mutex};
use cpal::{Sample, SampleRate};
use cpal::FromSample;

#[derive(Copy, Clone)]
pub enum Waveform {
    Sine,
    Square,
    Saw,
    Triangle,
}

#[derive(Clone)]
pub struct Oscillator {
    pub waveform: Waveform,
    pub current_sample_index: f32,
    pub last_sample: f32,
    pub frequency_hz: f32,
    pub amplitude: f32,
    pub phase_shift: f32,
    pub current_sample_rate: f32,
    pub inbox : Arc::<Mutex<Option<Receiver<String>>>>,
    pub phase : f32,
    pub phase_incr : f32
}

impl Oscillator {
    pub fn new (wave: Waveform, 
                sample_rate: SampleRate, 
                inbox: Receiver<String>, 
                freq: f32, 
                amp: f32
                ) -> Oscillator {
        return Oscillator {
            waveform: wave,
            current_sample_index: 0f32,
            last_sample: 0f32,
            frequency_hz: freq,
            amplitude: amp,
            phase_shift: 1f32,
            current_sample_rate: sample_rate.0 as f32,
            inbox : Arc::new(Mutex::new(Some(inbox))),
            phase : 0.0,
            phase_incr: 0.0
        }
    }

    // Ocsillator to buffer 
    pub fn process<'a, T>(&'a mut self) -> f32
    where
        T: Sample + FromSample<f32>,
    {
        self.update_increment();
        // ça c'est de la method de fumeur de mauvais shit 
        let inbox = Arc::clone(&self.inbox);
        let mut inbox = inbox.lock().unwrap();
        match inbox.as_mut().unwrap().try_recv() {
            Ok(msg) => self.check_inbox(msg),
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {panic!("inbox Disconnected !!")},
        }
        return self.next_sample();
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

    fn check_inbox (&mut self, msg: String) {
        /*
        Message syntaxe :
        parameter-value
        */
        let args: Vec<&str> = msg.trim().split('-').collect();

        let str_to_waveform = |arg: &str| match arg.to_lowercase().as_str() {
            "sine" => Waveform::Sine,
            "square" => Waveform::Square,
            "saw" => Waveform::Saw,
            "triangle" => Waveform::Triangle,
            _ => Waveform::Sine
        };

        match args[0] {
            "oscfreq" => self.frequency_hz = args[1].parse::<f32>().unwrap(),
            "osctype" => self.waveform = str_to_waveform(args[1]),
            _ => ()
        }
        
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