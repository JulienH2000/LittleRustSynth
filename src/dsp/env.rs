use crate::audiolib::*;
use crate::midi::*;
use std::sync::{Arc, Mutex};
use cpal::{
    FromSample, SizedSample,
};
use midir::MidiInput;

#[derive(Clone)]
pub struct Env {
    pub label: String,
    pub source: Option<Arc<Mutex<Node>>>,

}

impl Env {
    pub fn new (label: &str) -> Env {
        return Env {
            label: label.to_string(),
            source: None,
        }

    }
}

impl Processable for Env {
    fn process<T> (&mut self) -> f32 
    where
        T: SizedSample + FromSample<f32>,
    {

        let source: Arc<Mutex<Node>>;
        match &self.source {
            Some(src) => {
                source = src.clone();
            },
            None => panic!("Node oscillators!")
        }

        let buffer: f32;
        let envelop: f32 = 0.0;

        let source = Arc::clone(&source);
        let mut source = source.lock().unwrap();

        buffer = match &mut *source {
            Node::ModNode(md) => md.process::<T>() ,
            Node::OscNode(osc) => osc.process::<T>(),
            _ => panic!()
        } * envelop;

        return buffer;
    }
}

impl Routable for Env {
    fn route (&mut self, node: Arc<Mutex<Node>>) {
        self.source = Some(node);

    }
}