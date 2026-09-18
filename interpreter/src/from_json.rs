use serde_json::Value;

use crate::{Access, Cond, Instruction, Op, SimpleRef, StackType, StackValue};

pub trait FromJson {
    fn from_json(json: &Value) -> Self;
}

impl FromJson for u32 {
    fn from_json(json: &Value) -> u32 {
        let Value::Number(n) = json else {
            panic!("Invalid json")
        };
        n.as_u64().expect("Integer expected") as u32
    }
}

impl FromJson for i32 {
    fn from_json(json: &Value) -> i32 {
        let Value::Number(n) = json else {
            panic!("Invalid json")
        };
        n.as_i64().expect("Integer expected") as i32
    }
}

impl FromJson for StackValue {
    fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json["type"] else {
            panic!("Invalid json")
        };
        match s.as_str() {
            "int" => Self::Int(i32::from_json(&json["value"])),
            "integer" => Self::Int(i32::from_json(&json["value"])),
            "float" => todo!(),
            "ref" => todo!(),
            _ => panic!("Invalid json"),
        }
    }
}

impl FromJson for StackType {
    fn from_json(json: &Value) -> Self {
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

impl FromJson for Option<StackType> {
    fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json else {
            eprintln!("{}", json);
            if json.is_null() {
                return None;
            } else {
                panic!("Invalid json")
            }
        };
        match s.as_str() {
            "int" => Some(StackType::Int),
            "float" => Some(StackType::Float),
            "ref" => Some(StackType::Ref),
            _ => panic!("Invalid json"),
        }
    }
}

impl FromJson for Instruction {
    fn from_json(json: &Value) -> Self {
        let Value::String(s) = &json["opr"] else {
            panic!("Invalid json")
        };

        match s.as_str() {
            "ifz" => Self::Ifz {
                cond: Cond::from_json(&json["condition"]),
                target: u32::from_json(&json["target"]),
            },
            "if" => Self::If {
                cond: Cond::from_json(&json["condition"]),
                target: u32::from_json(&json["target"]),
            },
            "load" => Self::Load {
                ty: StackType::from_json(&json["type"]),
                index: u32::from_json(&json["index"]),
            },
            "store" => Self::Store {
                ty: StackType::from_json(&json["type"]),
                index: u32::from_json(&json["index"]),
            },
            "push" => Self::Push {
                value: StackValue::from_json(&json["value"]),
            },
            "return" => Self::Return {
                ty: Option::<StackType>::from_json(&json["type"]),
            },
            "get" => {
                if json["field"]["name"] != "$assertionsDisabled" {
                    todo!()
                }
                Self::Get
            }
            "new" => Self::New {
                class: json["class"].as_str().unwrap().to_string(),
            },
            "dup" => Self::Dup {
                words: json["words"].as_u64().unwrap() as u32,
            },
            "invoke" => Self::Invoke {
                access: Access::from_json(&json["access"]),
                method_name: json["method"]["name"]
                    .as_str()
                    .expect("Invalid json")
                    .to_string(),
                simple_ref: SimpleRef::from_json(&json["method"]["ref"]),
            },
            "throw" => Self::Throw,
            "binary" => Self::Binary {
                op: Op::from_json(&json["operant"]),
                ty: StackType::from_json(&json["type"]),
            },
            _ => unimplemented!("{}", json),
        }
    }
}

impl FromJson for Op {
    fn from_json(json: &Value) -> Self {
        match json.as_str().unwrap() {
            "add" => Self::Add,
            "sub" => Self::Sub,
            "mul" => Self::Mul,
            "div" => Self::Div,
            "rem" => Self::Rem,
            _ => panic!("Invalid json"),
        }
    }
}

impl FromJson for Access {
    fn from_json(json: &Value) -> Self {
        let access = json.as_str().expect("Invalid json");
        match access {
            "special" => Self::Special,
            "static" => todo!(),
            "dynamic" => todo!(),
            "interface" => todo!(),
            "virtual" => todo!(),
            _ => panic!("Invalid json"),
        }
    }
}

impl FromJson for SimpleRef {
    fn from_json(json: &Value) -> Self {
        let kind = json["kind"].as_str().expect("Invalid json");
        match kind {
            "class" => Self::Class {
                name: json["name"].as_str().expect("Invalid json").to_string(),
            },
            "array" => todo!(),
            _ => panic!("Invalid json"),
        }
    }
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
