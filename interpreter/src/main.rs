use std::{env, fs::File, io::BufReader};

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
                &methodname, &classname
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

impl Instruction {
    pub fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json["opr"] else {
            panic!("Invalid json")
        };

        match s.as_str() {
            "ifz" => Self::Ifz {
                cond: Cond::from_json(&json["condition"]),
                target: u32_from_json(&json["target"]),
            },
            "load" => Self::Load {
                ty: StackType::from_json(&json["type"]),
                index: u32_from_json(&json["index"]),
            },
            "store" => Self::Store {
                ty: StackType::from_json(&json["type"]),
                index: u32_from_json(&json["index"]),
            },
            "push" => Self::Push {
                value: StackValue::from_json(&json["value"]),
            },
            "return" => Self::Return {
                ty: StackType::option_from_json(&json["type"]),
            },
            "get" => Self::Get,
            "new" => Self::New {
                class: json["class"].as_str().unwrap().to_string(),
            },
            "dup" => Self::Dup {
                words: json["words"].as_u64().unwrap() as u32,
            },
            "invoke" => Self::Invoke,
            "throw" => Self::Throw,
            _ => unimplemented!("{}", json),
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

    pub fn option_from_json(json: &Value) -> Option<Self> {
        let Value::String(s) = &json else {
            eprintln!("{}", json);
            if json.is_null() {
                return None;
            } else {
                panic!("Invalid json")
            }
        };
        match s.as_str() {
            "int" => Some(Self::Int),
            "float" => Some(Self::Float),
            "ref" => Some(Self::Ref),
            _ => panic!("Invalid json"),
        }
    }
}

enum Cond {
    Ne,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Is,
    IsNot
}

impl Cond {
    pub fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json else {
            panic!("Invalid json")
        };
        match s.as_str() {
            "ne" => Self::Ne,
            "eq" => Self::Eq,
            "lt" => Self::Lt,
            "le" => Self::Le,
            "gt" => Self::Gt,
            "ge" => Self::Ge,
            "is" => Self::Is,
            "isnot" => Self::IsNot,
            _ => panic!("Invalid json: {}", json),
        }
    }
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
impl Either {
    fn to_sexp(&self) -> String {
        match self {
            Either::State(state) => state.to_sexp(),
            Either::Result(exe_result) => match exe_result {
                ExeResult::Ok => "ok".to_string(),
                ExeResult::AssertErr => "assertion error".to_string(),
                ExeResult::OutOfBounds => "out of bounds".to_string(),
                ExeResult::NullPointer => "null pointer".to_string(),
                ExeResult::Div0 => "divide by zero".to_string(),
                ExeResult::NoHalt => "*".to_string(),
            },
        }
    }
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
    fn to_sexp(&self) -> String {
        let mut res = "(state\n".to_string();

        //heap
        res += ":heap ";
        res += &self.heap.to_sexp();

        //frames
        res += ":frames (\n";
        for (i, frame) in self.frames.iter().enumerate() {
            res += &format!(":{:02} ", i);
            res += &frame.to_sexp();
        }
        res += ")\n";

        res += ")\n";
        res
    }

    fn new(program_counter: ProgramCounter) -> Self {
        Self {
            heap: Heap,
            frames: vec![Frame::new(program_counter)],
        }
    }
}

struct Heap;
impl Heap {
    fn to_sexp(&self) -> String {
        "()\n".to_string()
    }
}
struct Frame {
    stack: Vec<StackValue>,
    locals: Vec<Option<StackValue>>,
    program_counter: ProgramCounter,
}

impl Frame {
    fn to_sexp(&self) -> String {
        let mut res = "(frame\n".to_string();
        res += ":locals (\n";
        for (i, local) in self.locals.iter().enumerate() {
            res += &format!(":{:02} ", i);
            res += &StackValue::local_to_sexp(local);
        }
        res += ")\n";
        res += ":stack (\n";
        for (i, value) in self.stack.iter().enumerate() {
            res += &format!(":{:02} ", i);
            res += &value.to_sexp();
        }
        res += ")\n";
        res += ")\n";
        res
    }

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
impl ProgramCounter {
    fn to_sexp(&self) -> String {
        ":pc \"".to_string() + &self.0 + ":" + &self.1.to_string() + "\"\n"
    }
}

enum StackValue {
    Int(i32),
    Float(f32),
    Ref(u32),
}

impl StackValue {
    pub fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json["type"] else {
            panic!("Invalid json")
        };
        match s.as_str() {
            "integer" => Self::Int(i32_from_json(&json["value"])),
            "float" => todo!(),
            "ref" => todo!(),
            _ => panic!("Invalid json"),
        }
    }

    fn to_sexp(&self) -> String {
        let mut res = "(".to_string();
        match self {
            StackValue::Int(i) => {
                res += "int";
                res += &i.to_string();
            },
            StackValue::Float(f) => {
                res += "float";
                res += &f.to_string();
            },
            StackValue::Ref(r) => {
                res += "ref";
                res += &r.to_string();
            },
        }
        res += ")\n";
        res
    }

    fn local_to_sexp(local: &Option<Self>) -> String {
        if let Some(stack_val) = local {
            stack_val.to_sexp()
        } else {
            "(null)\n".to_string()
        }
    }
}

fn i32_from_json(json: &Value) -> i32 {
    let Value::Number(n) = json else {
        panic!("Invalid json")
    };
    n.as_i64().expect("Integer expected") as i32
}
