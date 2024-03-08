use std::{io, sync::{Arc, Mutex}};
use std::sync::mpsc::Receiver;
use cpal::Stream;
use crate::audiolib::*;
use std::collections::VecDeque;
use crate::dsp::oscillators::*;
use std::any::type_name;



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
    Invoke Node1>Node2>Node3>.... !Working
    See !Working
    Destroy !Not Working
    Edit Node !todo
    */

    //check command
    let args: Vec<&str> = msg.trim().split_whitespace().collect();
    match args[0].to_lowercase().as_str() {
        "invoke" => if args.len() > 1 {tree.invoke(args[1])} else { panic!("No arguments !!")},
        "see" => tree.see(),
        "destroy" => tree.destroy(),
        _ => panic!("Invalid command !! : {}", args[0]),
    };
}

//Currently Osc::new, future use
pub fn get_user_osc () -> Oscillator {
    let osc = Oscillator::new_empty();
    println!("oscillator set !");
    return osc;

}


//struct to fill with your nodes
#[derive(Clone)]
pub struct NodeTree {
   pub  nodes : Vec<Nodes>
}

impl NodeTree {

    pub fn new () -> NodeTree {
        return NodeTree { nodes : vec![] };
    }

    // Gets your message, translate your nodes to Nodes enum type, fills a NodeTree instance
    fn invoke (&mut self, nodes: &str) {
        println!("Building your tree...");
        self.nodes = vec![];
        let nodes: Vec<&str> = nodes.trim().split('>').collect();

        for node in nodes {
            match node {
                "oscnode" => {
                    self.nodes.push(Nodes::OscNode(None));
                },
                "modnode" => {
                    self.nodes.push(Nodes::ModNode(None));
                }
                "processnode" => {
                    self.nodes.push(Nodes::ProcessNode);
                },
                _ => panic!("Invalid Node !!")
            }
        }
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

        let mut tree: Vec<Nodes> = vec![];

        
        // Supports here 3 types of Node
        for node in &self.nodes {
            match node {
                Nodes::OscNode(osc) => match osc {
                    Some(osc) => tree.push(Nodes::OscNode(Some(osc.clone()))),
                    None => tree.push(Nodes::OscNode(Some(get_user_osc().context(host.clone(), inbox.clone()))))
                },
                Nodes::ModNode(oscmod) => match oscmod {
                    Some(oscmod) => tree.push(Nodes::ModNode(Some(oscmod.clone()))),
                    None => tree.push(Nodes::ModNode(Some(todo!())))
                },
                Nodes::ProcessNode => {tree.push(Nodes::ProcessNode); break;},
                _ => panic!("Invalid Node !!"), // No idea why this is unreachable..
            }
        }
        
        // reverse to parse from output to input
        tree = tree.into_iter().rev().collect();
        let mut tree = VecDeque::from_iter(tree);
        // Assure there is a proccess node at the end
        if let Nodes::ProcessNode = tree[0] {
            // removes process node
            tree.pop_front();
            let tree = Vec::from_iter(tree);
            let mut tree = tree.iter();
            // if next is osc, end parsing, as osc cannot take another node as input, the chain stops
            if let Some(Nodes::OscNode(osc)) = tree.next() {
                todo!() //compile & process the osc node
            }
            if let Some(Nodes::ModNode(oscmod)) = tree.next() {
                let oscs = tree.filter(|n| n == Nodes::OscNode(osc)).map(todo!(/*Unwrap oscillator */)).collect::<Vec<Oscillator>>();
                match oscmod { Some( oscmod ) => oscmod.populate(oscs), _ => panic!("mod empty !!")};
                todo!(/*process oscmod */)
                
            }
        }

        // Check if your tree is empty ! 
        if self.nodes.len() > 1 {
            println!("Compile tree...");
            let stream = ProcessNode::new(self.nodes[0].clone(), host).make::<f32>();
            return Some(stream);
        } else {
            //Empty tree ! No compile and continue...
            return None;
        }
        

    }

}