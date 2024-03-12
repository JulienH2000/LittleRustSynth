use core::panic;
use std::{io, sync::{Arc, Mutex}};
use std::sync::mpsc::Receiver;
use cpal::Stream;
use crate::{audiolib::*, dsp::modulation::OscModulator};
use std::collections::VecDeque;
use crate::dsp::oscillators::*;
use std::any::TypeId;
use std::collections::BTreeMap;

//simple user input reading
pub fn get_user_input() -> String {
    let mut user_input = String::new();
            io::stdin()
                .read_line(&mut user_input)
                .expect("failed");
    user_input
}

//User command interpreter
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

//Spawn new user-defined oscillator
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

pub fn get_user_mod () -> OscModulator {
    // label, index
    println!("define new modulator :");
    let uip = get_user_input();
    let vec_uip = uip.trim().split(',').collect::<Vec<&str>>();
    if uip.len() == 2 {
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

    /*
    // Gets your message, translate your nodes to Node enum type, fills a NodeTree instance
    fn invoke (&mut self, nodes: &str) {
        println!("Building your tree...");
        self.nodes = vec![];
        let nodes: Vec<&str> = nodes.trim().split('>').collect();

        for node in nodes {
            match node {
                "oscnode" => {
                    self.nodes.push(Node::OscNode(None));
                },
                "modnode" => {
                    self.nodes.push(Node::ModNode(None));
                }
                "processnode" => {
                    self.nodes.push(Node::ProcessNode);
                },
                _ => panic!("Invalid Node !!")
            }
        }
    }
    */

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

    // Just display a NodeTree, in order
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
    pub fn compile (&mut self, host: Arc<Mutex<Option<HostConfig>>>, inbox: Arc<Mutex<Option<Receiver<String>>>> ) -> Option<Stream> {

        let mut compile_tree: Vec<Node> = vec![];

        /*
        // Supports here 3 types of Node
        // Construct a new compilable tree, filling the nodes whith concrete instances
        for node in &self.nodes {
            match node {
                Node::OscNode(osc) => match osc {
                    Some(osc) => compile_tree.push(Node::OscNode(Some(osc.clone()))),
                    None => compile_tree.push(Node::OscNode(Some(get_user_osc().context(host.clone(), inbox.clone()))))
                }, 
                Node::ModNode(oscmod) => match oscmod {
                    Some(oscmod) => compile_tree.push(Node::ModNode(Some(oscmod.clone()))),
                    None => compile_tree.push(Node::ModNode(Some(OscModulator::new(inbox, mod_index))))
                },
                Node::ProcessNode => {compile_tree.push(Node::ProcessNode); break;},
                _ => panic!("Invalid Node !!"), // No idea why this is unreachable..
            }
        }
        
        // reverse to parse from output to input
        compile_tree = compile_tree.into_iter().rev().collect();
        let mut compile_tree = VecDeque::from_iter(compile_tree);
        // Assure there is a proccess node at the end
        if let Node::ProcessNode = compile_tree[0] {
            // removes process node
            compile_tree.pop_front();
            let compile_tree = Vec::from_iter(compile_tree);
            let mut compile_tree = compile_tree.iter();
            // if next is osc, end parsing, as osc cannot take another node as input, the chain stops
            if let Some(Node::OscNode(osc)) = compile_tree.next() {
                todo!() //compile & process the osc node
            }
            if let Some(Node::ModNode(oscmod)) = compile_tree.next() {
                //lookup next nodes, filter osc, unwrap, collect and send to processing
                let oscs = compile_tree.filter(|n| matches!(n, Node::OscNode(_))).map(|n| ).collect::<Vec<Oscillator>>();
                match oscmod { Some( oscmod ) => oscmod.populate(oscs), _ => panic!("mod empty !!")};
                todo!(/*process oscmod */)
                
            }
        }

        */
        // Generate Node table, with label fetching capacity
        let mut node_table: BTreeMap<String, Node> = BTreeMap::new();
        for node in &self.nodes {
            match node {
                Node::OscNode(osc) => {node_table.insert(osc.label.to_owned(), Node::OscNode(osc.context(host.clone(), inbox.clone())));},
                Node::ModNode(oscmod) => {node_table.insert(oscmod.label.to_owned(), Node::ModNode(oscmod.clone()));},
                Node::ProcessNode => {node_table.insert("Default_PN".to_string(), Node::ProcessNode);}
            }
        }

        let mut process_node = ProcessNode::new(None, host);

        // Route parsing
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
        //println!("je process cette merde!");
        if self.nodes.len() > 1 {
            let stream = process_node.make::<f32>();
            return Some(stream);
        } else {
            return None;
        }



    


/*
        // Check if your tree is empty ! 
        if self.nodes.len() > 1 {
            println!("Compile tree...");
            let stream = ProcessNode::new(self.nodes[0].clone(), host).make::<f32>();
            return Some(stream);
        } else {
            //Empty tree ! No compile and continue...
            return None;
        }
*/
        

    }

}


pub fn route_node<T> (source: Node, dest: &mut T) 
where
    T: Routable,
    {
    dest.route(source);
}
