use crate::audiolib::*;
use std::sync::{Arc, Mutex};
use cpal::{
    FromSample, SizedSample,
};


#[derive(Clone)]
pub struct OscModulator {
    pub label: String,
    pub oscs: Vec<Arc<Mutex<Node>>>,
    pub mod_index : f32
}

impl OscModulator {

    pub fn new (label: String, mod_index: f32) -> OscModulator {

        return OscModulator {
            label : label,
            oscs : vec![],
            mod_index : mod_index
        }
    }

    pub fn process<T> (&mut self) -> f32 
    where
        T: SizedSample + FromSample<f32>,
    {

        if self.oscs.len() == 0 {
            panic!("Node oscillators!");
        }

        let mut buffer: f32 = 0.0;

        for osc in &mut self.oscs {

            let osc = Arc::clone(&osc);
            let mut osc = osc.lock().unwrap();
            let osc = match &mut *osc {
                Node::OscNode(osc) => osc,
                _ => panic!()
            };
            buffer *= self.mod_index;
            buffer += osc.process::<T>();
        }
        return buffer;
    }

}

impl Routable for OscModulator {
    fn route (&mut self, node: Arc<Mutex<Node>>) {
        self.oscs.push(node);

    }
}