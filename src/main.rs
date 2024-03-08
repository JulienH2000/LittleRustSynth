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

 let _audio_thread = thread::spawn(move || {

println!("running audio thread...");
let mut tree = NodeTree::new();
let mut stream_vec: Vec<Stream> = vec![];

    loop {
        //println!("Running !");
        let rx = Arc::clone(&rx);
        let rxp = Arc::clone(&rxp);
        let host = Arc::clone(&host);
        //println!("Receive MSG...");
        let receive = rx.lock().unwrap();
        match receive.try_recv() {
            Ok(new_tree) => tree = new_tree,
            Err(TryRecvError::Empty) => tree = tree,
            Err(TryRecvError::Disconnected) => {panic!("Threads disconnected !!")},
        };
        //println!("Try compile received tree...");


        if stream_vec.len() == 0 {
            match tree.compile(host, rxp) {
                Some(stream) => stream_vec.push(stream),
                None => continue
            }
        } 
        //println!("Running Stream !");
        stream_vec[0].play().unwrap()

        /*
        let option_stream = tree.compile(host, rxp);
        match &option_stream {
            Some(_s) => println!("Some Stream !"),
            None => {}//println!("No Stream !")
        }
        let stream = match option_stream {
        Some(new_stream) => new_stream,
        None => continue
        };
        println!("Running Stream !");
        stream.play().unwrap()
        */
        }
});

let mut tree = NodeTree::new();


let msg = RwLock::new("".to_string());

loop {
    {
        println!("Listening to user command...");
        let mut w_msg = msg.write().unwrap();
        *w_msg = get_user_input();
    }

    {
        let r_msg = msg.read().unwrap();
        let r_msg = r_msg.clone();
        //println!("Parse user command... is:{}", r_msg);
        get_user_command(r_msg, &mut tree);


    }

let msg = "osctype-sine&&oscfreq-440".to_string();

//println!("Try streaming tree...");
let tx = Arc::clone(&tx);
let txp = Arc::clone(&txp);
let transmit = tx.lock().unwrap();
transmit.send(tree.clone()).unwrap();

let post = txp.lock().unwrap();
post.send(msg).unwrap();

}


//audio_thread.join().unwrap();



////////////////////////////////////////////////


/*

// Init message channel
let (tx, rx) = channel();
let tx = Arc::new(Mutex::new(tx));
let rx = Arc::new(Mutex::new(rx));

// User Interface //
    let tx = Arc::clone(&tx);
    loop {
        let transmit = tx.lock().unwrap();
        println!("Listening to user command...");
        let msg = get_user_input();
        //let msg = Arc::new(Mutex::new(msg));
        transmit.send(msg).unwrap();
    }

            // Audio //

    // Init Host
    let mon_host = HostConfig::new();
    let _mon_sample_rate = mon_host.config.sample_rate;
    let mut tree = NodeTree::new();
    let host = Arc::new(Mutex::new(Some(mon_host)));
    println!("running audio...");

    let rx = Arc::clone(&rx);
        loop {
            println!("prout");
            let receive = rx.lock().unwrap();
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


    


    _ui_thread.join().unwrap();
    _audio_thread.join().unwrap();


//ma_sortie.play().unwrap();
//let message = get_user_input();
//tx.send(message).unwrap();

*/
    
}







