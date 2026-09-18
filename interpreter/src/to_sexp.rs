use crate::{Either, ExeResult, Frame, Heap, ProgramCounter, StackValue, State};

pub trait ToSexp {
    fn to_sexp(&self) -> String;
}

impl ToSexp for StackValue {
    fn to_sexp(&self) -> String {
        let mut res = "(".to_string();
        match self {
            StackValue::Int(i) => {
                res += "int ";
                res += &i.to_string();
            }
            StackValue::Float(f) => {
                res += "float ";
                res += &f.to_string();
            }
            StackValue::Ref(r) => {
                res += "ref ";
                res += &r.to_string();
            }
        }
        res += ")\n";
        res
    }
}

impl ToSexp for Option<StackValue> {
    fn to_sexp(&self) -> String {
        if let Some(stack_val) = self {
            stack_val.to_sexp()
        } else {
            "(null)\n".to_string()
        }
    }
}

impl ToSexp for ProgramCounter {
    fn to_sexp(&self) -> String {
        ":pc \"".to_string() + &self.0 + ":" + &self.1.to_string() + "\"\n"
    }
}

impl ToSexp for Frame {
    fn to_sexp(&self) -> String {
        let mut res = "(frame\n".to_string();
        res += ":locals (\n";
        for (i, local) in self.locals.iter().enumerate() {
            res += &format!(":{:02} ", i);
            res += &local.to_sexp();
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
}

impl ToSexp for Heap {
    fn to_sexp(&self) -> String {
        "()\n".to_string()
    }
}

impl ToSexp for State {
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
}

impl ToSexp for Either {
    fn to_sexp(&self) -> String {
        match self {
            Either::State(state) => state.to_sexp(),
            Either::Result(exe_result) => match exe_result {
                ExeResult::Ok => "ok".to_string(),
                ExeResult::AssertErr => "\"assertion error\"".to_string(),
                ExeResult::OutOfBounds => "\"out of bounds\"".to_string(),
                ExeResult::NullPointer => "\"null pointer\"".to_string(),
                ExeResult::Div0 => "\"divide by zero\"".to_string(),
                ExeResult::NoHalt => "\"*\"".to_string(),
            },
        }
    }
}
