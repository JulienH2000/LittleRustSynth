use core::panic;
use std::{io, os::windows::process, rc::Rc, sync::{Arc, Mutex}};
use std::sync::mpsc::Receiver;
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
   pub nodes : BTreeMap<String, Arc<Mutex<Node>>>
}

impl NodeTree {

    pub fn new () -> NodeTree {
        let mut new = NodeTree { nodes : BTreeMap::new() };
        new.nodes.insert("process".to_string(), Arc::new(Mutex::new(Node::ProcessNode(ProcessNode::new(None, None)))));
        return new
    }

    /*
    fn _add (&mut self, msg: &str) {
        let (cmd, arg) = msg.trim().split_once(' ').unwrap();
        match cmd.to_lowercase().as_str() {
            "route" => self.new_route(arg),
            "node" => self.new_node(arg),
            _ => panic!("Unresolved \"add\" argument "),
        }

    }

    fn _new_node (&mut self, msg: &str) {
        let node = msg.trim();
        match node.to_lowercase().as_str() {
            "oscnode" => {
                self.nodes.push(Node::OscNode(get_user_osc()));
            },
            "modnode" => {
                self.nodes.push(Node::ModNode(get_user_mod()));
            },
            "processnode" => {
                if self.nodes.iter().filter(|n| if let Node::ProcessNode = n {return true} else {return false}).count() == 0 {
                    self.nodes.push(Node::ProcessNode);
                } else {
                    panic!("Only one ProcessNode can be suumoned in a tree !!")
                }

            },
            _ => panic!("Unresolved Node type !!")
        }

    }

    fn _new_route (&mut self, msg: &str) {
        let (source, destination) = msg.trim().split_once('>').unwrap();
        self.routes.push(Route::new(source, destination));

    }
    */


    pub fn new_osc (&mut self, label: String, wave: Waveform, freq: f32, amp: f32) {
        // label,type,freq,amp
        let osc = Oscillator::new(label, wave, None, freq, amp);
        self.nodes.insert(osc.label.clone(), Arc::new(Mutex::new(Node::OscNode(osc))));

    }

    pub fn new_mod (&mut self, label: String, index: f32) {
        let modulator = OscModulator::new(label, index);
        self.nodes.insert(modulator.label.clone(), Arc::new(Mutex::new(Node::ModNode(modulator))));
    }

    pub fn route_in (&mut self, source_label: String, dest_label: String) {
        let src = self.nodes.get(&source_label).unwrap();
        let dest = self.nodes.get(&dest_label).unwrap();
        let inner_dest = Arc::clone(&dest);
        let mut inner_dest = dest.lock().unwrap();
        match &mut *inner_dest {
            Node::OscNode(_osc) => panic!(),
            Node::ModNode(oscmod) => route_node(src.clone(), oscmod),
            Node::ProcessNode(process) => route_node(src.clone(), process),
            _ => panic!()
        }
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

    // Compile your tree into a cpal::Stream, return a Option, to return None if your tree is empty
    // Cool things happening here..
    /*
    pub fn compile (&mut self, host: Arc<Mutex<Option<HostConfig>>>) -> Option<Stream> {

        // Generate Node table, with label fetching capacity
        // Push audio context if needed
        let mut node_table: BTreeMap<String, Node> = BTreeMap::new();
        for node in &self.nodes {
            match node {
                Node::OscNode(osc) => {node_table.insert(osc.label.to_owned(), Node::OscNode(osc.context(host.clone())));},
                Node::ModNode(oscmod) => {node_table.insert(oscmod.label.to_owned(), Node::ModNode(oscmod.clone()));},
                Node::ProcessNode => {node_table.insert("Default_PN".to_string(), Node::ProcessNode);}
            }
        }

        // The Default, Unique, ProcessNode, empty
        // Must me added by user ! 
        let mut process_node = ProcessNode::new(None, host);

        // Route parsing
        // Parse each route, fetch the node in the node base by label, and route the source into the destination
        for route in &self.routes {
            let src: Node;
            let dst: &mut Node;
            {
                src = match node_table.get(&route.source) {
                    Some(src) => src.clone(),
                    None => panic!(),
                };
            }
            {
                dst = match node_table.get_mut(&route.destination) {
                    Some(dst) => dst,
                    None => panic!(),
                };  
            }
            match dst {
                Node::OscNode(_osc) => panic!(),
                Node::ModNode(oscmod) => route_node(src, oscmod),
                Node::ProcessNode => route_node(src, &mut process_node),
                _ => panic!()
            }
            
        }
        // if a node has been routed into the ProcessNode, Run the audio !!
        if let Some(_) = process_node.input_node  {
            let stream = process_node.make::<f32>();
            return Some(stream);
        } else {
            return None;
        }
        
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
