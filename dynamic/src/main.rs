use std::{env, fmt::Display};

use interpreter::*;
use itertools::Itertools;
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "info" {
        println!("Rust analyzer (not that one)\n0.1.0\nGroup Rust\nrust,dynamic\n");
        return;
    }
    let (classname, methodname) = get_class_and_method(&args[1]);
    let json: Value = read_json(&classname);
    let class = Class::from_json(&json);

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
        if !method.id.params.is_empty() {
            println!("ok;skip");
            println!("divide by zero;skip");
            println!("assertion error;skip");
            println!("out of bounds;skip");
            println!("null pointer;skip");
            println!("*;skip");
            return;
        }
    }
    let consts: Vec<_> = class
        .methods
        .iter()
        .map(|m| {
            m.instructions.iter().filter_map(|inst| match inst {
                Instruction::Push { value } => Some(value.clone()),
                _ => None,
            })
        })
        .flatten()
        .collect();
    let possible_inputs: Vec<Vec<_>> = method
        .id
        .params
        .iter()
        .map(|param| {
            let mut inputs = input_for_param(param);
            let ty = inputs[0].get_type();
            inputs.extend(consts.iter().filter(|c| c.get_type() == ty));
            inputs
        })
        .collect();

    let input_iter = possible_inputs
        .into_iter()
        .map(|vec| vec.into_iter())
        .multi_cartesian_product();

    let mut pred = Prediction {
        ok: "no".to_string(),
        div0: "no".to_string(),
        assert_err: "no".to_string(),
        out_of_bounds: "no".to_string(),
        null_pointer: "no".to_string(),
        no_halt: "no".to_string(),
    };

    let interpreter = Interpreter::new(class);
    for input in input_iter {
        let (_, res) = interpreter.interpret(&method, (input, Heap { heap: vec![] }), 1000);
        match res {
            ExeResult::Ok => pred.ok = "yes".to_string(),
            ExeResult::AssertErr => pred.assert_err = "yes".to_string(),
            ExeResult::OutOfBounds => pred.out_of_bounds = "yes".to_string(),
            ExeResult::NullPointer => pred.null_pointer = "yes".to_string(),
            ExeResult::Div0 => pred.div0 = "yes".to_string(),
            ExeResult::NoHalt => pred.no_halt = "yes".to_string(),
            ExeResult::DidNotFinish => {
                if pred.no_halt != "yes" {
                    pred.no_halt = "probably".to_string()
                }
            }
        }
    }
    println!("{}", pred)
}

fn input_for_param(param: &SimpleType) -> Vec<StackValue> {
    match param {
        SimpleType::Int => (-100..100).map(|i| StackValue::Int(i)).collect(),
        SimpleType::Float => todo!(),
        SimpleType::Byte => (0..256).map(|i| StackValue::Int(i)).collect(),
        SimpleType::Char => (0..256).map(|i| StackValue::Int(i)).collect(),
        SimpleType::Short => (-100..100).map(|i| StackValue::Int(i)).collect(),
        SimpleType::Boolean => (0..=1).map(|i| StackValue::Int(i)).collect(),
        SimpleType::SimpleRef(_) => todo!(),
    }
}

struct Prediction {
    ok: String,
    div0: String,
    assert_err: String,
    out_of_bounds: String,
    null_pointer: String,
    no_halt: String,
}

impl Display for Prediction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ok;{}", self.ok)?;
        writeln!(f, "divide by zero;{}", self.div0)?;
        writeln!(f, "assertion error;{}", self.assert_err)?;
        writeln!(f, "out of bounds;{}", self.out_of_bounds)?;
        writeln!(f, "null pointer;{}", self.null_pointer)?;
        write!(f, "*;{}", self.no_halt)?;
        Ok(())
    }
}
