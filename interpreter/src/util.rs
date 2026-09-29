use std::{fs::File, io::BufReader};

use regex::Regex;
use serde_json::Value;

use crate::{
    Heap, SimpleType, StackValue,
    abstractions::{IntAbstraction, IntLike},
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

pub fn empty_input<T: IntLike>() -> (Vec<StackValue<T>>, Heap<T>) {
    (vec![], Heap { heap: vec![] })
}

pub fn abstract_input<T: IntAbstraction>(params: &[SimpleType]) -> (Vec<StackValue<T>>, Heap<T>) {
    let heap = Heap { heap: vec![] };
    let mut locals = Vec::with_capacity(params.len());

    for p in params {
        match p {
            SimpleType::Int => locals.push(StackValue::Int(T::new_int())),
            SimpleType::Float => todo!(),
            SimpleType::Byte => todo!(),
            SimpleType::Char => todo!(),
            SimpleType::Short => todo!(),
            SimpleType::Boolean => locals.push(StackValue::Int(T::new_bool())),
            SimpleType::SimpleRef(_) => todo!(),
        }
    }

    (locals, heap)
}
