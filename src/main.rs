use std::{sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex, RwLock}, thread};
use std::sync::mpsc::TryRecvError;
pub mod audiolib;
pub mod dsp;
pub mod toolbox;
use cpal::{traits::StreamTrait, Stream};

use crate::audiolib::*;
use crate::toolbox::*;


fn main() {

// Command channel
let (tx, rx): (Sender<NodeTree>, Receiver<NodeTree>) = channel();
let tx = Arc::new(Mutex::new(tx));
let rx = Arc::new(Mutex::new(rx));
// User parameter channel
let (txp, rxp): (Sender<String>, Receiver<String>) = channel();
let txp = Arc::new(Mutex::new(txp));
let rxp = Arc::new(Mutex::new(Some(rxp)));


// Init Host
let mon_host = HostConfig::new();
let _mon_sample_rate = mon_host.config.sample_rate;
let host = Arc::new(Mutex::new(Some(mon_host)));

// Use a dedicated thread, to keep stream.play() in scope
let _audio_thread = thread::spawn(move || {

println!("running audio thread...");
let mut tree = NodeTree::new();
let mut stream_vec: Vec<Stream> = vec![];

    loop {
        // Atomic Refs 
        let rx = Arc::clone(&rx);
        let rxp = Arc::clone(&rxp);
        let host = Arc::clone(&host);
        // Unwrap the tree Receiver
        let receive = rx.lock().unwrap();
        match receive.try_recv() {
            Ok(new_tree) => tree = new_tree,
            Err(TryRecvError::Empty) => tree = tree,
            Err(TryRecvError::Disconnected) => {panic!("Threads disconnected !!")},
        };

        // Check if there is a current working Stream, else create a new stream
        if stream_vec.len() == 0 {
            //Check the tree, if empty loop back, if full compile it to stream
            match tree.compile(host, rxp) {
                Some(stream) => stream_vec.push(stream),
                None => continue
            }
        } 
        // Running the Stream ! Keep it in scope !
        stream_vec[0].play().unwrap();
        }
});

let mut tree = NodeTree::new();
// Read/Write lock to the tree message
let msg = RwLock::new("".to_string());

loop {
    {
        // Lock, get the input, Write
        println!("Listening to user command...");
        let mut w_msg = msg.write().unwrap();
        *w_msg = get_user_input();
    }

    {
        // Lock, Read, Return a tree
        let r_msg = msg.read().unwrap();
        let r_msg = r_msg.clone();
        //println!("Parse user command... is:{}", r_msg);
        get_user_command(r_msg, &mut tree);


    }

// Default OSC parameters, while the parameters send is WIP
let msg = "".to_string();


// Send Tree to audio thread
let tx = Arc::clone(&tx);
let txp = Arc::clone(&txp);
let transmit = tx.lock().unwrap();
transmit.send(tree.clone()).unwrap();

// Send Osc parameters to audio thread
let post = txp.lock().unwrap();
post.send(msg).unwrap();

}


//audio_thread.join().unwrap();
}