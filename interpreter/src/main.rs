#![allow(unused)]
use std::{env, fs::File, io::BufReader};

use regex::Regex;
use serde_json::Value;

mod to_sexp;
use to_sexp::ToSexp;
mod from_json;
use from_json::FromJson;
mod java_types;
use java_types::*;
mod instruction;
use instruction::*;
mod state;
use state::*;
mod java_class;
use java_class::*;

use crate::ExeResult::{AssertErr, Div0, Ok};

fn get_class_and_method(arg: &str) -> (String, String) {
    let re = Regex::new(r"(.*)\.(.*):(.*)").unwrap();
    let caps = re.captures(arg).unwrap();

    let classname = caps.get(1).unwrap().as_str().replace('.', "/");
    let methodname = caps.get(2).unwrap().as_str().to_string();
    (classname, methodname)
}

fn read_json(classname: &str) -> Value {
    let path = "target/decompiled/".to_string() + classname + ".json";
    eprintln!("path {}", path);
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "info" {
        println!("Rust analyzer (not that one)\n0.1.0\n\nrust,syntactic\n");
        return;
    }
    let (classname, methodname) = get_class_and_method(&args[1]);
    let input = &args[2];
    let iter: u32 = args[3].parse().unwrap();
    let json: Value = read_json(&classname);
    let class = Class::from_json(&json);
    let method = class
        .methods
        .iter()
        .find(|m| m.id.name == methodname)
        .expect(&format!(
            "Method {} should be implemented on {}",
            methodname, classname
        ));
    let input = create_input(method, input);
    interpret(&args[1], method, input, iter);
}

fn create_input(method: &Method, input: &str) -> Vec<StackValue> {
    let types = &method.id.params;

    let splitted_input = input[1..input.len() - 1].split(",").map(|s| s.trim());
    types
        .iter()
        .zip(splitted_input)
        .map(|(t, input)| {
            dbg!(input);
            match t {
                SimpleType::Int => StackValue::Int(input.parse().unwrap()),
                SimpleType::Float => todo!(),
                SimpleType::Byte => todo!(),
                SimpleType::Char => todo!(),
                SimpleType::Short => todo!(),
                SimpleType::Boolean => {
                    let b: bool = input.parse().unwrap();
                    StackValue::Int(b as i32)
                }
                SimpleType::SimpleRef(simple_ref) => todo!(),
            }
        })
        .collect()
}

fn interpret(abs_method_name: &str, method: &Method, input: Vec<StackValue>, iter: u32) {
    let pc = ProgramCounter(abs_method_name.to_string(), 0);
    let mut state = State::new(pc, input);
    println!("(init {} )", state.to_sexp());
    for _ in 0..iter {
        println!("(step\n:before {}", state.to_sexp());
        let (pc, res) = step(&method.instructions, state);
        println!("{}", pc.to_sexp());
        println!(":after {}", res.to_sexp());
        println!(")");
        state = if let Either::State(s) = res {
            s
        } else {
            return;
        }
    }
}

fn step(bytecode: &[Instruction], mut state: State) -> (ProgramCounter, Either) {
    let Some(mut cur_frame) = state.frames.pop() else {
        panic!("Empty state!")
    };
    dbg!(&bytecode[cur_frame.pc()]);
    match &bytecode[cur_frame.pc()] {
        Instruction::Load { ty, index } => cur_frame.load(*ty, *index),
        Instruction::Push { value } => cur_frame.push(*value),
        Instruction::Dup { words } => cur_frame.dup(*words),
        Instruction::Throw => return (cur_frame.program_counter, Either::Result(AssertErr)),
        Instruction::Ifz { cond, target } => {
            let i = match cur_frame.stack.pop() {
                Some(StackValue::Int(i)) => i as i64,
                Some(StackValue::Ref(i)) => i as i64,
                Some(_) => panic!(),
                None => panic!(),
            };
            if cond.cmp_with(i, 0) {
                cur_frame.set_pc(*target);
                let pc = cur_frame.program_counter.clone();
                state.frames.push(cur_frame);
                return (pc, Either::State(state));
            }
        }
        Instruction::Store { ty, index } => cur_frame.store(*ty, *index),
        Instruction::Return { ty } => {
            if state.frames.is_empty() {
                return (cur_frame.program_counter, Either::Result(Ok));
            }
            todo!();
        }
        Instruction::Get => {
            // TODO handle other cases than assertionsDisabled = false
            cur_frame.stack.push(StackValue::Int(0));
            dbg!(&cur_frame.stack);
        }
        Instruction::New { class } => {
            cur_frame
                .stack
                .push(StackValue::Ref(state.heap.heap.len() as u32));
            state
                .heap
                .heap
                .push(SimpleType::SimpleRef(Box::new(SimpleRef::Class {
                    name: class.clone(),
                })));
        }
        Instruction::Invoke {
            access,
            method_id,
            simple_ref,
        } => {
            // TODO actual stuff

            cur_frame.stack.pop();
        }
        Instruction::Binary { op, ty } => match ty {
            StackType::Int => {
                let Some(StackValue::Int(rhs)) = cur_frame.stack.pop() else {
                    panic!()
                };
                let Some(StackValue::Int(lhs)) = cur_frame.stack.pop() else {
                    panic!()
                };
                dbg!(lhs, rhs);
                match op.op_int(lhs, rhs) {
                    Some(result) => cur_frame.stack.push(StackValue::Int(result)),
                    None => return (cur_frame.program_counter, Either::Result(Div0)),
                }
            }
            StackType::Float => todo!(),
            StackType::Ref => panic!("Cannot use ref for arithmetic operations!"),
        },
        Instruction::If { cond, target } => {
            let rhs = match cur_frame.stack.pop() {
                Some(StackValue::Int(i)) => i as i64,
                Some(StackValue::Ref(i)) => i as i64,
                Some(_) => panic!(),
                None => panic!(),
            };
            let lhs = match cur_frame.stack.pop() {
                Some(StackValue::Int(i)) => i as i64,
                Some(StackValue::Ref(i)) => i as i64,
                Some(_) => panic!(),
                None => panic!(),
            };

            if cond.cmp_with(lhs, rhs) {
                cur_frame.set_pc(*target);
                let pc = cur_frame.program_counter.clone();
                state.frames.push(cur_frame);
                return (pc, Either::State(state));
            }
        }
        Instruction::Goto { target } => {
            cur_frame.set_pc(*target);
            let pc = cur_frame.program_counter.clone();
            state.frames.push(cur_frame);
            return (pc, Either::State(state));
        }
        Instruction::Placeholder => todo!(),
    }
    cur_frame.increment_pc();
    let pc = cur_frame.program_counter.clone();
    state.frames.push(cur_frame);
    (pc, Either::State(state))
}
