use std::{fs::File, io::BufReader};

use regex::Regex;
use serde_json::Value;

use crate::{
    Heap, SimpleType, StackValue, Type, abstractions::{IntAbstraction, IntLike, Like},
};

pub fn get_class_and_method(arg: &str) -> (String, String) {
    let re = Regex::new(r"(.*)\.(.*):(.*)").unwrap();
    let caps = re.captures(arg).unwrap();

    let classname = caps.get(1).unwrap().as_str().replace('.', "/");
    let methodname = caps.get(2).unwrap().as_str().to_string();
    (classname, methodname)
}

pub fn read_json(classname: &str) -> Value {
    let path = "target/decompiled/".to_string() + classname + ".json";
    eprintln!("path {}", path);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

pub fn empty_input<I:IntLike, F:Like, R:Like>() -> (Vec<StackValue<I,F,R>>, Heap<I,F,R>) {
    (vec![], Heap { heap: vec![] })
}

pub fn abstract_input<I: IntAbstraction, F: Like, R: Like>(params: &[Type]) -> (Vec<StackValue<I,F,R>>, Heap<I,F,R>) {
    let heap = Heap { heap: vec![] };
    let mut locals = Vec::with_capacity(params.len());

    for p in params {
        match p {
            Type::S(SimpleType::Int) => locals.push(StackValue::Int(I::new_int())),
            Type::S(SimpleType::Boolean) => locals.push(StackValue::Int(I::new_bool())),
            _ => todo!(),
        }
    }

    (locals, heap)
}
