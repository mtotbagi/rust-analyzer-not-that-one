use std::{env, fs::File, io::BufReader, ops::Deref};

use regex::Regex;
use serde_json::Value::{self, Array};

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
    assert!(args.len() == 2);
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
        v.iter().find(|method| {
            method["name"] == methodname
        }).expect(&format!("Method {} should be implemented for {}", &methodname, &classname))
    } else {
        panic!("Invalid json");
    };

    interpret(method, input, iter);

    // eprintln!("Found method: {}", method);

    // if let Array(instructions) = &method["code"]["bytecode"] {
    //     for inst in instructions {
    //         eprintln!("{}", inst)
    //     }
    // } else {
    //     panic!("Invalid json")
    // }
}

enum Instruction {
    Ifz {cond: Cond, target: u32 },
    Load {ty: StackType, index: u32},
    Push {ty: StackType, index: u32},
    Return {ty: Option<StackType>}
}

impl Instruction {
    pub fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json["opr"] else {
            panic!("Invalid json")
        };

        match s.as_str() {
            "ifz" => Self::Ifz { cond: Cond::from_json(&json["condition"]), target: u32_from_json(&json["target"]) },
            _ => unimplemented!()
        }
    }
}

fn u32_from_json(json: &Value) -> u32 {
    let Value::Number(n) = json else {
        panic!("Invalid json")
    };
    n.as_u64().expect("Integer expected") as u32
}

enum StackType {
    Int,
    Float,
    Ref,
}

impl StackType {
    pub fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json else {
            panic!("Invalid json")
        };
        match s.as_str() {
            "int" => Self::Int,
            "float" => Self::Float,
            "ref" => Self::Ref,
            _ => panic!("Invalid json"),
        }
    }
}

enum Cond {
    Ne,
    Eq,
}

impl Cond {
    pub fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json["opr"] else {
            panic!("Invalid json")
        };
        match s.as_str() {
            "ne" => Self::Ne,
            "eq" => Self::Eq,
            _ => unimplemented!(),
        }
    }
}

fn interpret(method: &Value, input: &str, iter: u32) {
    let instructions: Vec<_> = if let Array(instructions) = &method["code"]["bytecode"] {
        instructions.iter().map(Instruction::from_json).collect()
    } else {
        panic!("Invalid json")
    };

}

fn step(bytecode: Vec<Instruction>, state: State) -> (ProgramCounter, Either) {
    todo!()
}

enum Either {
    State(State),
    Result(ExeResult)
}
enum ExeResult {
    Ok,
    AssertErr,
    OutOfBounds,
    NullPointer,
    Div0,
    NoHalt
}

struct State {
    heap: Heap,
    frames: Vec<Frame>,
}

struct Heap {}
struct Frame {
    stack: Vec<StackValue>,
    locals: Vec<Option<StackValue>>,
    program_counter: (String, u32),
}

struct ProgramCounter(String, u32);

enum StackValue {
    Int(i32),
    Float(f32),
    Ref(u32),
}
