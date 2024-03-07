use std::{io, sync::{Arc, Mutex}};
use cpal::{
    traits::{DeviceTrait, HostTrait}, FromSample, SampleRate, SizedSample
};
use crate::audiolib::*;
use crate::dsp::oscillators::*;



pub fn get_user_input() -> String {
    let mut user_input = String::new();
            io::stdin()
                .read_line(&mut user_input)
                .expect("failed");
    user_input
}

pub fn get_user_command (msg: String, tree: &mut NodeTree) {
    /*
    Syntaxe :
    Invoke Node1>Node2>Node3>....
    See
    Destroy
    Edit Node
    */
    println!("Tree building command :\n");


    //check command
    let args: Vec<&str> = msg.trim().split_whitespace().collect();
    match args[0].to_lowercase().as_str() {
        "invoke" => tree.invoke_nodes(args[1]),
        "see" => tree.see_nodes(),
        "destroy" => {},
        _ => {},
    };
}

fn get_user_osc () -> (Waveform, f32, f32) {
    println!("Set parameter to invoke oscillator !\nType,Frequency,Amplitude");
    let input = get_user_input();
    let args: Vec<&str> = input.trim().split(',').collect();

    let str_to_waveform = |arg: &str| match arg.to_lowercase().as_str() {
        "sine" => Waveform::Sine,
        "square" => Waveform::Square,
        "saw" => Waveform::Saw,
        "triangle" => Waveform::Triangle,
        _ => Waveform::Sine
    };
    let wavetype = str_to_waveform(args[0]);
    let freq = args[2].parse::<f32>().unwrap();
    let amp = args[3].parse::<f32>().unwrap();

    return (wavetype, freq, amp);

}


#[derive(Clone)]
pub struct NodeTree {
   pub  nodes : Vec<Nodes>
}

impl NodeTree {

    pub fn new () -> NodeTree {
        return NodeTree { nodes : vec![] };
    }

    fn invoke_nodes (&mut self, nodes: &str, ) {
        let nodes: Vec<&str> = nodes.trim().split('>').collect();

        for node in nodes {
            match node {
                "oscnode" => {
                    let options = get_user_osc();
                    self.nodes.push(Nodes::OscNode(Oscillator::new(options.0, None, None, options.1, options.2)));
                },
                "processnode" => {
                    self.nodes.push(Nodes::ProcessNode);
                },
                _ => {}
            }
        }
    }

    pub fn see_nodes (&self) {
        let nodes = &self.nodes;
        for node in nodes {
            println!("->");
            println!("{}", node);
        }
    }

}