use std::{collections::HashMap, process::{Command, Output}, sync::{mpsc::channel, Arc, Mutex, RwLock}, thread};
use std::sync::mpsc::{Receiver, TryRecvError, SendError, TrySendError, Sender};
pub mod audiolib;
pub mod dsp;
pub mod toolbox;
use cpal::traits::StreamTrait;

use crate::audiolib::*;
use crate::dsp::oscillators::*;
use crate::toolbox::*;

fn main() {

// Init message channel
let (tx, rx) = channel();
let tx = Arc::new(Mutex::new(tx));
let rx = Arc::new(Mutex::new(rx));

            // Audio //
    let _audio_thread = thread::spawn(move || {
    // Init Host
    let mon_host = HostConfig::new();
    let mon_sample_rate = mon_host.config.sample_rate;
    let mut tree = NodeTree::new();
    let host = Arc::new(Mutex::new(Some(mon_host)));
    println!("running audio...");

    let rx = Arc::clone(&rx);
    let receive = rx.lock().unwrap();
        loop {
            
            //let command = &rx;
            match receive.try_recv() {
                Ok(msg) => get_user_command(msg, &mut tree),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {panic!("Threads disconnected !!")},
            }


            let host = Arc::clone(&host);
            let stream = tree.compile(host, Arc::new(Mutex::new(None)));
            println!("Try streaming tree...");
            stream.play().unwrap();
        }
    }
    );


    // User Interface //
    let _ui_thread = thread::spawn(move || {
    let tx = Arc::clone(&tx);
    let transmit = tx.lock().unwrap();
        loop {
            println!("Listening to user command...");
            let msg = get_user_input();
            //let msg = Arc::new(Mutex::new(msg));
            transmit.send(msg).unwrap();
        }
    }
    );


    _ui_thread.join().unwrap();
    _audio_thread.join().unwrap();


//ma_sortie.play().unwrap();
//let message = get_user_input();
//tx.send(message).unwrap();

    
}







