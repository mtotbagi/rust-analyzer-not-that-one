#![allow(unused)]
use std::{env, fs::File, io::BufReader};

use regex::Regex;
use serde_json::Value::{self, Array};

mod to_sexp;
use to_sexp::ToSexp;
mod from_json;
use from_json::FromJson;

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
    let methods = &json["methods"];
    let method = if let Array(v) = methods {
        v.iter()
            .find(|method| method["name"] == methodname)
            .expect(&format!(
                "Method {} should be implemented for {}",
                methodname, classname
            ))
    } else {
        panic!("Invalid json");
    };

    interpret(&args[1], method, input, iter);
}

enum Instruction {
    Ifz { cond: Cond, target: u32 },
    Load { ty: StackType, index: u32 },
    Store { ty: StackType, index: u32 },
    Push { value: StackValue },
    Return { ty: Option<StackType> },
    Get,
    New { class: String },
    Dup { words: u32 },
    Invoke,
    Throw,
}

enum StackType {
    Int,
    Float,
    Ref,
}

enum Cond {
    Ne,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Is,
    IsNot,
}

fn interpret(abs_method_name: &str, method: &Value, input: &str, iter: u32) {
    let instructions: Vec<_> = if let Array(instructions) = &method["code"]["bytecode"] {
        instructions.iter().map(Instruction::from_json).collect()
    } else {
        panic!("Invalid json")
    };
    let pc = ProgramCounter(abs_method_name.to_string(), 0);
    let mut state = State::new(pc);
    println!("(init {} )", state.to_sexp());
    for _ in 0..iter {
        println!("(step\n:before {}", state.to_sexp());
        let (pc, res) = step(&instructions, state);
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

fn step(bytecode: &[Instruction], state: State) -> (ProgramCounter, Either) {
    assert!(!state.frames.is_empty());
    (
        state.frames[0].program_counter.clone(),
        Either::State(state),
    )
}

enum Either {
    State(State),
    Result(ExeResult),
}

enum ExeResult {
    Ok,
    AssertErr,
    OutOfBounds,
    NullPointer,
    Div0,
    NoHalt,
}

struct State {
    heap: Heap,
    frames: Vec<Frame>,
}

impl State {
    fn new(program_counter: ProgramCounter) -> Self {
        Self {
            heap: Heap,
            frames: vec![Frame::new(program_counter)],
        }
    }
}

struct Heap;
struct Frame {
    stack: Vec<StackValue>,
    locals: Vec<Option<StackValue>>,
    program_counter: ProgramCounter,
}

impl Frame {
    fn new(program_counter: ProgramCounter) -> Self {
        Self {
            stack: vec![],
            locals: vec![],
            program_counter,
        }
    }
}

#[derive(Clone)]
struct ProgramCounter(String, u32);

enum StackValue {
    Int(i32),
    Float(f32),
    Ref(u32),
}
