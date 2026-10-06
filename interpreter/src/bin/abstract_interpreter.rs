use std::env;

use interpreter::*;
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "info" {
        println!("Rust analyzer (not that one)\n0.1.0\nGroup Rust\nrust,dynamic\n");
        return;
    }
    let (classname, methodname) = get_class_and_method(&args[1]);
    let json: Value = read_json(&classname);
    let class: Class<SignSet> = Class::from_json(&json);

    let method = class
        .get_method(&methodname)
        .expect(&format!(
            "Method {} should be implemented on {}",
            methodname, classname
        ))
        .clone();
    if method
        .id
        .params
        .iter()
        .any(|param| matches!(param, SimpleType::SimpleRef(_)))
    {
        panic!();
    }

    let interpreter = Interpreter::new(class);
    let (states, results) = interpreter.abstract_interpret(&method, 100);
    let pcs: Vec<ProgramCounter> = states
        .iter()
        .flat_map(|s| s.iter().map(|state| state.program_counter()))
        .collect();
    let state = states[0][0].clone();
    eprintln!("{results:?}");
    println!("(init {} )", state.to_sexp());
    for pc in &pcs {
        println!("(step\n");
        println!("{}", pc.to_sexp());
        println!(":edit (update\n");
        println!(":path () \n");
        println!(":a {}", state.to_sexp());
        println!(":b {}", state.to_sexp());
        println!("))");
    }
    for res in results {
        println!("(step\n");
        println!("{}", pcs[0].to_sexp());
        println!(":edit (update\n");
        println!(":path () \n");
        println!(":a {}", state.to_sexp());
        println!(":b {}", res.to_sexp());
        println!("))");
    }
}

fn vec_to_sexp<T: IntLike>(states: &Vec<State<T>>) -> String {
    let mut result = "(states\n".to_string();
    for s in states {
        result += &s.to_sexp();
    }
    result += ")\n";
    result
}
