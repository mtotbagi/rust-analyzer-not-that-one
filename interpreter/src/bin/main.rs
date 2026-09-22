use std::env;
use serde_json::Value;

use interpreter::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "info" {
        println!("Rust analyzer (not that one)\n0.1.0\nGroup Rust\nrust,syntactic\n");
        return;
    }
    let (classname, methodname) = get_class_and_method(&args[1]);
    let input = &args[2];
    let iter: u32 = args[3].parse().unwrap();
    let json: Value = read_json(&classname);
    let class = Class::from_json(&json);
    let method = class.get_method(&methodname)
        .expect(&format!(
            "Method {} should be implemented on {}",
            methodname, classname
        ))
        .clone();
    let input = parse_input(&method, input);
    let interpreter = Interpreter::new(class);
    let (states, res) = interpreter.interpret(&method, input, iter);

    println!("(init {} )", states[0].to_sexp());
    for i in 0..states.len() {
        if i == states.len() - 1 && res == ExeResult::DidNotFinish {
            continue;
        }
        let state = &states[i];
        println!("(step\n:before {}", state.to_sexp());
        let pc = state.program_counter();
        println!("{}", pc.to_sexp());
        if i == states.len() - 1 {
            println!(":after {}", res.to_sexp());
        } else {
            println!(":after {}", states[i + 1].to_sexp());
        }
        println!(")");
    }
}