use std::{cell::RefCell, rc::Rc, sync::Arc};

use rust_lisp::model::{Env, RuntimeError, Value};

use crate::{TREE, RECOMPILE_FLAG};
use crate::dsp::oscillators::Waveform;

//pub type NativeFunc = fn(env: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError>;

pub fn lisp_new_osc (_env: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError> {
    //(new_osc "name" "wave" freq amp)

    let tree = Arc::clone(&TREE);
    let mut tree = tree.lock().unwrap();

    let label = match &args[0] {
        Value::String(s) => s,
        _ => {return Err(RuntimeError { msg: "Invalid string".to_string() })}
    };
    let wave = match &args[1] {
        Value::String(s) => s,
        _ => {return Err(RuntimeError { msg: "Invalid string".to_string() })}
    };
    let freq = match args[2] {
        Value::Float(f) => f,
        Value::Int(i) => i as f32,
        _ => {return Err(RuntimeError { msg: "Invalid float".to_string() })}
    };
    let amp = match args[3] {
        Value::Float(f) => f,
        Value::Int(i) => i as f32,
        _ => {return Err(RuntimeError { msg: "Invalid float".to_string() })}
    };
 
    let label: String = label.to_string();
    let wave: Waveform = match wave.to_lowercase().as_str() {
            "sine" => Waveform::Sine, 
            "square" => Waveform::Square,
            "saw" => Waveform::Saw,
            "triangle" => Waveform::Triangle,
            _ => panic!("Unresolved Waveform !!")
        };
    let freq: f32 = freq;
    let amp: f32 = amp;

    tree.new_osc(label, wave, freq, amp);
    
    /*
    let flag = Arc::clone(&RECOMPILE_FLAG);
    let mut flag = flag.lock().unwrap();
    *flag = true;
    */

    return Ok(Value::String("Oscillator added !".to_string()));
}

pub fn lisp_edit_osc_freq (_env: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError> {
    //(new_osc "name" "wave" freq amp)

    let tree = Arc::clone(&TREE);
    let mut tree = tree.lock().unwrap();

    let label = match &args[0] {
        Value::String(s) => s,
        _ => {return Err(RuntimeError { msg: "Invalid string".to_string() })}
    };
    let freq = match args[1] {
        Value::Float(f) => f,
        Value::Int(i) => i as f32,
        _ => {return Err(RuntimeError { msg: "Invalid freq".to_string() })}
    };
 
    let label: &str = label;
    let freq: f32 = freq;

    tree.edit_osc_freq(label, freq);
    
    /*
    let flag = Arc::clone(&RECOMPILE_FLAG);
    let mut flag = flag.lock().unwrap();
    *flag = true;
    */

    return Ok(Value::String("Oscillator edited !".to_string()));
}

pub fn lisp_new_mod (_env: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError> {
    //(new_mod "name" mod_index)
    
    let tree = Arc::clone(&TREE);
    let mut tree = tree.lock().unwrap();

    let label = match &args[0] {
        Value::String(s) => s,
        _ => {return Err(RuntimeError { msg: "Invalid string".to_string() })}
    };
    let index = match args[1] {
        Value::Float(f) => f,
        Value::Int(i) => i as f32,
        _ => {return Err(RuntimeError { msg: "Invalid float".to_string() })}
    };
 
    let label: String = label.to_string();
    let index: f32 = index;


    tree.new_mod(label, index);

    /*
    let flag = Arc::clone(&RECOMPILE_FLAG);
    let mut flag = flag.lock().unwrap();
    *flag = true;
    */

    return Ok(Value::String("Modulator added !".to_string()));
}

pub fn lisp_route_node (_env: Rc<RefCell<Env>>, args: Vec<Value>) -> Result<Value, RuntimeError> {
    //(new_osc "name" "wave" freq amp)
    
    let tree = Arc::clone(&TREE);
    let mut tree = tree.lock().unwrap();

    let source = match &args[0] {
        Value::String(s) => s,
        _ => {return Err(RuntimeError { msg: "Invalid string".to_string() })}
    };
    let destination = match &args[1] {
        Value::String(s) => s,
        _ => {return Err(RuntimeError { msg: "Invalid string".to_string() })}
    };

 
    let src_label: String = source.to_string();
    let dest_label: String = destination.to_string();

    

    tree.route_in(src_label, dest_label);


    let flag = Arc::clone(&RECOMPILE_FLAG);
    let mut flag = flag.lock().unwrap();
    *flag = true;


    return Ok(Value::String("Nodes routed !".to_string()));
}

pub fn lisp_clear_process (_env: Rc<RefCell<Env>>, _args: Vec<Value>) -> Result<Value, RuntimeError> {

    let tree = Arc::clone(&TREE);
    let mut tree = tree.lock().unwrap();

    tree.clear();

    let flag = Arc::clone(&RECOMPILE_FLAG);
    let mut flag = flag.lock().unwrap();
    *flag = true;


    return Ok(Value::String("Process cleared !".to_string()));
}
