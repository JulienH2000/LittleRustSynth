use core::time;
use std::{error::Error, sync::{ mpsc::{channel, Receiver}, Arc, Mutex, RwLock}, thread};
use cpal::{traits::StreamTrait, Stream};
use lisp_bindings::{lisp_clear_process, lisp_edit_osc_freq, lisp_midi_ports, lisp_new_mod, lisp_route_node};
use rust_lisp::{default_env, start_repl};
use rust_lisp::model::{Symbol, Value};
use lazy_static::lazy_static;

pub mod audiolib;
pub mod dsp;
pub mod toolbox;
pub mod lisp_bindings;
pub mod midi;
pub mod syerr;
use crate::audiolib::*;
use crate::toolbox::*;
use crate::lisp_bindings::lisp_new_osc;

lazy_static!{
    pub static ref TREE: Arc<Mutex<NodeTree>>= Arc::new(Mutex::new(NodeTree::new()));
    pub static ref RECOMPILE_FLAG: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    pub static ref MIDI_MASTER_INDEX: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None)); 

}

fn main() -> Result<(), Box<dyn Error>> {

{
// Midi listenning init
let _ = midi::display_inputs()?;
println!("Choose midi input (by index):");
let user_usize = toolbox::get_user_input();
let user_usize = user_usize.trim().parse::<usize>()?;
let midi_master_index = Arc::clone(&MIDI_MASTER_INDEX);
let mut midi_master_index = midi_master_index.lock().unwrap();
*midi_master_index = Some(user_usize);
}

let (midi_tx, midi_rx) = channel::<midi::MidiMessage>();

let t_midi_tx = Arc::new(Mutex::new(midi_tx));
let t_midi_rx = Arc::new(Mutex::new(midi_rx));

let _midi_thread = thread::spawn(
    
    move || {
        let mut index: Option<usize> = None;
        let mut midi_master: Option<midi::MidiModule> = None;
            let midi_master_index = Arc::clone(&MIDI_MASTER_INDEX);
            let midi_master_index = midi_master_index.lock().unwrap();
            match *midi_master_index {
                Some(new_index) => {
                    if let None = index { index = Some(new_index) };
                },
                None => panic!()
            };

            match midi_master {
                Some(_) => {
            
                },
                None => {
                    if let Some(i) = index {
                        midi_master = Some(midi::MidiModule::new(i).unwrap());
                    }
                },
            }

            let _message = match midi_master {
                Some(m) => m.listen(t_midi_tx).unwrap(),
                None => panic!()
            };

            thread::sleep(time::Duration::from_millis(10));

        }

);



// Init audio Host
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
            let t_midi_rx = Arc::clone(&t_midi_rx);

            //Check the tree, if empty loop back, if full compile it to stream
            let flag = Arc::clone(&RECOMPILE_FLAG);
            let mut flag = flag.lock().unwrap();
            if *flag == true {
            match tree.compile(host, t_midi_rx) {
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
env.define(
    Symbol::from("midi?"),
    Value::NativeFunc(lisp_midi_ports)
);
  
start_repl(Some(env));




    Ok(())
}
