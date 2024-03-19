use std::{sync::{ Arc, Mutex}, thread};
use cpal::{traits::StreamTrait, Stream};
use lisp_bindings::{lisp_clear_process, lisp_edit_osc_freq, lisp_new_mod, lisp_route_node};
use rust_lisp::{default_env, start_repl};
use rust_lisp::model::{Symbol, Value};
use lazy_static::lazy_static;

pub mod audiolib;
pub mod dsp;
pub mod toolbox;
pub mod lisp_bindings;
use crate::audiolib::*;
use crate::toolbox::*;
use crate::lisp_bindings::lisp_new_osc;

lazy_static!{
    pub static ref TREE: Arc<Mutex<NodeTree>>= Arc::new(Mutex::new(NodeTree::new()));
    pub static ref RECOMPILE_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
}

fn main() {

// Init Host
let mon_host = HostConfig::new();
let _mon_sample_rate = mon_host.config.sample_rate;
let host = Arc::new(Mutex::new(mon_host));



// Use a dedicated thread, to keep stream.play() in scope
let _audio_thread = thread::spawn(move || {

    //println!("running audio thread...");
    let mut stream: Vec<Stream> = vec![];

        loop {
            // Atomic Refs 
            let host = Arc::clone(&host);
            let tree = Arc::clone(&TREE);
            let mut tree = tree.lock().unwrap();

            //Check the tree, if empty loop back, if full compile it to stream
            let flag = Arc::clone(&RECOMPILE_FLAG);
            let mut flag = flag.lock().unwrap();
            if *flag == true {
            match tree.compile(host) {
                Some(s) => {
                    *flag = false;
                    stream.push(s)
                },
                None => {
                    *flag = false;
                    if stream.len() != 0 { stream.pop();}
                    continue
                }
            }
            }

            // Running the Stream ! Keep it in scope !
            match stream.last() {
                Some(s) => s.play().unwrap(),
                None => continue
            }

            //println!("{}", flag);
            
            }
});

let mut env = default_env();

env.define(
    Symbol::from("osc"),
    Value::NativeFunc(lisp_new_osc)
  );
env.define(
    Symbol::from("mod"),
    Value::NativeFunc(lisp_new_mod)
);
env.define(
    Symbol::from("route"),
    Value::NativeFunc(lisp_route_node)
);
env.define(
    Symbol::from("edit_freq"),
    Value::NativeFunc(lisp_edit_osc_freq)
);
env.define(
    Symbol::from("clear"),
    Value::NativeFunc(lisp_clear_process)
);

  
start_repl(Some(env));







//audio_thread.join().unwrap();
}
