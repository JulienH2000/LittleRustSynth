use std::{io, sync::{mpsc::channel, Arc, Mutex, RwLock}};
use std::sync::mpsc::{Receiver, TryRecvError};
use cpal::{
    traits::{DeviceTrait, HostTrait}, FromSample, SampleRate, SizedSample, Stream};
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

    //check command
    let args: Vec<&str> = msg.trim().split_whitespace().collect();
    match args[0].to_lowercase().as_str() {
        "invoke" => if args.len() > 1 {tree.invoke(args[1])} else { panic!("No arguments !!")},
        "see" => tree.see(),
        "destroy" => {},
        _ => panic!("Invalid command !!"),
    };
}

pub fn get_user_osc () -> Oscillator {
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
    let freq = args[1].parse::<f32>().unwrap();
    let amp = args[2].parse::<f32>().unwrap();

    let osc = Oscillator::new(wavetype, None, None, freq, amp);
    println!("Parameters set !");
    return osc;

}


#[derive(Clone)]
pub struct NodeTree {
   pub  nodes : Vec<Nodes>
}

impl NodeTree {

    pub fn new () -> NodeTree {
        return NodeTree { nodes : vec![] };
    }

    fn invoke (&mut self, nodes: &str) {
        println!("Building your tree...");
        self.nodes = vec![];
        let nodes: Vec<&str> = nodes.trim().split('>').collect();

        for node in nodes {
            match node {
                "oscnode" => {
                    self.nodes.push(Nodes::OscNode(None));
                },
                "processnode" => {
                    self.nodes.push(Nodes::ProcessNode);
                },
                _ => panic!("Invalid Node !!")
            }
        }
    }

    pub fn see (&self) {
        let nodes = &self.nodes;
        for node in nodes {
            println!("->{}", node);
        }
    }

    pub fn compile (&mut self, host: Arc<Mutex<Option<HostConfig>>>, inbox: Arc<Mutex<Option<Receiver<String>>>> ) -> Stream {
        println!("Compile tree...");

        let mut buf: Vec<Nodes> = vec![];

        for node in &self.nodes {
            match node {
                Nodes::OscNode(osc) => match osc {
                    Some(osc) => buf.push(Nodes::OscNode(Some(osc.clone()))),
                    None => buf.push(Nodes::OscNode(Some(get_user_osc().context(host.clone(), inbox.clone()))))
                },
                Nodes::ProcessNode => {buf.push(Nodes::ProcessNode); break;},
                _ => panic!("Invalid Node !!"),
            }
        }

        self.nodes = buf;

        let stream = ProcessNode::new(self.nodes[0].clone(), host).make::<f32>();
        stream
    }

}