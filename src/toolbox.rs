use core::panic;
use std::{io, sync::{Arc, Mutex}};
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

// User command interpreter
pub fn get_user_command (msg: String, tree: &mut NodeTree) {
    /*
    Syntaxe :
    add > Node "node" / Route "source>dest"
    remove > Node / Route
    See !Working
    Destroy !Not Working
    Edit Node !todo
    */

    //check command
    let (cmd, arg) = msg.trim().split_once(' ').unwrap();
    match cmd.to_lowercase().as_str() {
        "add" => if cmd.len() > 1 {tree.add(arg)} else { panic!("No arguments !!")},
        "remove" => todo!(), //if cmd.len() > 1 {tree.remove(arg)} else { panic!("No arguments !!")},
        "see" => tree.see(),
        "destroy" => tree.destroy(),
        _ => panic!("Invalid command !! : {}", cmd),
    };
}

// Spawn new user-defined oscillator
pub fn get_user_osc () -> Oscillator {
    // label,type,freq,amp
    println!("define new oscillator :");
    let uip = get_user_input();
    let vec_uip = uip.trim().split(',').collect::<Vec<&str>>();
    if vec_uip.len() == 4 {
        let label: String = vec_uip[0].to_string();
        let wave: Waveform = match vec_uip[1].to_lowercase().as_str() {
            "sine" => Waveform::Sine,
            "square" => Waveform::Square,
            "saw" => Waveform::Saw,
            "triangle" => Waveform::Triangle,
            _ => panic!("Unresolved Waveform !!")
        };
        let freq: f32 = vec_uip[2].parse().unwrap();
        let amp: f32 = vec_uip[3].parse().unwrap();

        let osc = Oscillator::new(label, wave, None, None, freq, amp);

        return osc

    } else {
        panic!("unresolved arguments !!");
    }

}

// Spawn new user-defined modulator
pub fn get_user_mod () -> OscModulator {
    // label, index
    println!("define new modulator :");
    let uip = get_user_input();
    let vec_uip = uip.trim().split(',').collect::<Vec<&str>>();
    if vec_uip.len() == 2 {
        let label = vec_uip[0].to_string();
        let mod_index = vec_uip[1].parse::<f32>().unwrap();

        let oscmod = OscModulator::new(label, None, mod_index);
        return oscmod

    } else {
        panic!("unresolved arguments !!");
    }
    

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
   pub nodes : Vec<Node>,
   pub routes : Vec<Route>
}

impl NodeTree {

    pub fn new () -> NodeTree {
        return NodeTree { nodes : vec![], routes: vec![] };
    }

    fn add (&mut self, msg: &str) {
        let (cmd, arg) = msg.trim().split_once(' ').unwrap();
        match cmd.to_lowercase().as_str() {
            "route" => self.new_route(arg),
            "node" => self.new_node(arg),
            _ => panic!("Unresolved \"add\" argument "),
        }

    }

    fn new_node (&mut self, msg: &str) {
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

    fn new_route (&mut self, msg: &str) {
        let (source, destination) = msg.trim().split_once('>').unwrap();
        self.routes.push(Route::new(source, destination));

    }

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

    // Compile your tree into a cpal::Stream, return a Option, to return None if your tree is empty
    // Cool things happening here..
    pub fn compile (&mut self, host: Arc<Mutex<Option<HostConfig>>>, inbox: Arc<Mutex<Option<Receiver<String>>>> ) -> Option<Stream> {

        // Generate Node table, with label fetching capacity
        // Push audio context if needed
        let mut node_table: BTreeMap<String, Node> = BTreeMap::new();
        for node in &self.nodes {
            match node {
                Node::OscNode(osc) => {node_table.insert(osc.label.to_owned(), Node::OscNode(osc.context(host.clone(), inbox.clone())));},
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
            println!("c'est parti !");
            let stream = process_node.make::<f32>();
            return Some(stream);
        } else {
            return None;
        }
        
    }

}

// Generic for all Nodes
pub fn route_node<T> (source: Node, dest: &mut T) 
where
    T: Routable,
    {
    dest.route(source);
}
