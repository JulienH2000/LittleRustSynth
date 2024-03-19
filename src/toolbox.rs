use core::panic;
use std::{io, sync::{Arc, Mutex}};
use cpal::Stream;
use crate::{audiolib::*, dsp::modulation::OscModulator};
use crate::dsp::oscillators::*;
use std::collections::BTreeMap;

// simple user input reading
pub fn get_user_input() -> String {
    let mut user_input = String::new();
            io::stdin()
                .read_line(&mut user_input)
                .expect("failed");
    user_input
}

#[derive(Clone)]
pub struct Route {
    pub source : String,
    pub destination : String,
}

// Route contains a list of input-output crosspoints, as label strings couple
impl Route {

    pub fn new (source: &str, destination: &str) -> Route {
        Route {
            source: source.to_string(),
            destination: destination.to_string()
        }
    }

}


//struct to fill with your nodes
#[derive(Clone)]
pub struct NodeTree {
   pub nodes : BTreeMap<String, Arc<Mutex<Node>>>,
   pub flag : bool
}

impl NodeTree {

    pub fn new () -> NodeTree {
        let mut new = NodeTree { nodes : BTreeMap::new(), flag: false };
        new.nodes.insert("process".to_string(), Arc::new(Mutex::new(Node::ProcessNode(ProcessNode::new(None, None)))));
        return new
    }

    pub fn new_osc (&mut self, label: String, wave: Waveform, freq: f32, amp: f32) {
        // label,type,freq,amp
        let osc = Oscillator::new(label, wave, None, freq, amp);
        self.nodes.insert(osc.label.clone(), Arc::new(Mutex::new(Node::OscNode(osc))));

    }

    pub fn edit_osc_freq (&mut self, label: &str, freq: f32) {
        // label,type,freq,amp
        let node = self.nodes.get_mut(label).unwrap();
        let node = Arc::clone(&node);
        let mut node = node.lock().unwrap();
        match &mut *node {
            Node::OscNode(osc) => osc.frequency_hz = freq,
            _ => panic!()
        }
    }

    pub fn new_mod (&mut self, label: String, index: f32) {
        let modulator = OscModulator::new(label, index);
        self.nodes.insert(modulator.label.clone(), Arc::new(Mutex::new(Node::ModNode(modulator))));
    }

    pub fn route_in (&mut self, source_label: String, dest_label: String) {
        let src = self.nodes.get(&source_label).unwrap();
        let dest = self.nodes.get(&dest_label).unwrap();
        //let inner_dest = Arc::clone(&dest);
        let mut inner_dest = dest.lock().unwrap();
        match &mut *inner_dest {
            Node::OscNode(_osc) => panic!(),
            Node::ModNode(oscmod) => route_node(src.clone(), oscmod),
            Node::ProcessNode(process) => route_node(src.clone(), process),
            //_ => panic!()
        }
    }

    pub fn clear (&mut self) {
        let process_node = self.nodes.get("process").unwrap();
        let process_node = Arc::clone(&process_node);
        let mut process_node = process_node.lock().unwrap();
        let process_node = match &mut *process_node {
            Node::ProcessNode(process) => process,
            _ => panic!()
        };

        process_node.input_node = None;
    }

    pub fn compile (&mut self, host: Arc<Mutex<HostConfig>>) -> Option<Stream> {
        for (_label, node) in self.nodes.iter() {
            let node_clone = Arc::clone(&node);
            let mut node_clone = node_clone.lock().unwrap();
            match &mut *node_clone {
                Node::OscNode(osc) => {osc.context(host.clone());},
                Node::ModNode(_oscmod) => {},
                Node::ProcessNode(process) => {process.context(host.clone());}
            }
        }

        // if a node has been routed into the ProcessNode, Run the audio !!
        let process_node = self.nodes.get("process").unwrap();
        let process_node = Arc::clone(&process_node);
        let mut process_node = process_node.lock().unwrap();
        let process_node = match &mut *process_node {
            Node::ProcessNode(process) => process,
            _ => panic!()
        };
        if let Some(_) = process_node.input_node  {
            let stream = process_node.make::<f32>();
            return Some(stream);
        } else {
            return None;
        }
    }

    /*
    // Just display a NodeTree, in order, NOT WORKING
    pub fn see (&self) {
        let nodes = &self.nodes;
        for node in nodes {
            println!("->{}", node);
        }
    }

    // Destroy your tree, 
    // not working due to compile ignoring empty tree if a previous working stream exists
    pub fn destroy (&mut self) {
        *self = NodeTree::new();
    }
    */

}

// Generic for all Nodes
pub fn route_node<T> (source: Arc<Mutex<Node>>, dest: &mut T) 
where
    T: Routable,
    {
    dest.route(source);
}
