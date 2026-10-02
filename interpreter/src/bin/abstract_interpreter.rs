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
        return;
    }

    let interpreter = Interpreter::new(class);
    let results = interpreter.abstract_interpret(&method, 100);
    for res in results {
        eprintln!("{:?}", res)
    }
}
