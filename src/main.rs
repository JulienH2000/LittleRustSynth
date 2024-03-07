use std::{sync::{mpsc::channel, Arc, Mutex}, thread};
pub mod audiolib;
pub mod dsp;
pub mod toolbox;
use crate::audiolib::*;
use crate::dsp::oscillators::*;
use crate::toolbox::*;

fn main() {
    // Init Host
    let mon_host = HostConfig::new();
    let mon_sample_rate = mon_host.config.sample_rate;

    //let mon_oscillateur = SourceNode::OscNode(Oscillator::new(Waveform::Triangle, mon_sample_rate, rx, 440_f32, 1f32));
    //let mut mon_stream = ProcessNode::new(mon_oscillateur, mon_host);
    //let ma_sortie = mon_stream.make::<f32>();

    let mut empty_tree = NodeTree::new();
    let tree = Arc::new(Mutex::new(empty_tree));
    let edit_tree = Arc::clone(&tree);
    let process_tree = Arc::clone(&tree);

    // User Interface Thread
    let interface_thread = thread::spawn(move || {

        loop {
            let mut tree = edit_tree.lock().unwrap();
            let input = get_user_input();
            get_user_command(input, &mut tree);
            
        }  
    });

    // Audio Thread
    let audio_thread = thread::spawn(move || {

        // Init message channel
       // let (tx, rx) = channel();
        // Init Host
        let mon_host = HostConfig::new();
        let mon_sample_rate = mon_host.config.sample_rate;



        loop {
            let mut tree = process_tree.lock().unwrap();
            let mut node1 = Oscillator::new_empty();
            

            for node in &mut tree {
                match node {
                    Nodes::OscNode(osc) => {
                        node = Nodes::OscNode(node.context(mon_sample_rate, None));
                    },
                    Nodes::ProcessNode => {
                        let stream = ProcessNode::new(node1, mon_host);
                    },
                    _ => panic!("Invalid Node !!")
                }
            }





            //ma_sortie.play().unwrap();
            //let message = get_user_input();
            //tx.send(message).unwrap();
        }

    });

    interface_thread.join().unwrap();
    audio_thread.join().unwrap();
    
    
    
}







