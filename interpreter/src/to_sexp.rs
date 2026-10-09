use crate::{
    Either, ExeResult, Frame, Heap, ProgramCounter, StackValue, State,
    abstractions::{IntLike, Like},
};

pub trait ToSexp {
    fn to_sexp(&self) -> String;
}

impl<I, F, R> ToSexp for StackValue<I, F, R> {
    fn to_sexp(&self) -> String {
        let mut res = "(".to_string();
        match self {
            StackValue::Int(_) => {
                res += "int ()";
                // res += &i.to_string();
            }
            StackValue::Float(_) => {
                res += "float ()";
                // res += &f.to_string();
            }
            StackValue::Ref(_) => {
                res += "ref ()";
                // res += &r.to_string();
            }
        }
        res += ")\n";
        res
    }
}

impl<I, F, R> ToSexp for Option<StackValue<I, F, R>> {
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
        let classname = self.class.replace("/", ".");
        ":pc \"".to_string()
            + &classname
            + "."
            + &self.method.to_string()
            + ":"
            + &self.idx.to_string()
            + "\"\n"
    }
}

impl<I: IntLike, F: Like, R: Like> ToSexp for Frame<I, F, R> {
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

impl<I: IntLike, F: Like, R: Like> ToSexp for Heap<I, F, R> {
    fn to_sexp(&self) -> String {
        "()\n".to_string()
    }
}

impl<I: IntLike, F: Like, R: Like> ToSexp for State<I, F, R> {
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

impl ToSexp for ExeResult {
    fn to_sexp(&self) -> String {
        match self {
            ExeResult::Ok => "ok".to_string(),
            ExeResult::AssertErr => "\"assertion error\"".to_string(),
            ExeResult::OutOfBounds => "\"out of bounds\"".to_string(),
            ExeResult::NullPointer => "\"null pointer\"".to_string(),
            ExeResult::Div0 => "\"divide by zero\"".to_string(),
            ExeResult::NoHalt => "\"*\"".to_string(),
            ExeResult::DidNotFinish => panic!("cannot make DidNotFinish a sexp"),
        }
    }
}

impl<I: IntLike, F: Like, R: Like> ToSexp for Either<I, F, R> {
    fn to_sexp(&self) -> String {
        match self {
            Either::State(state) => state.to_sexp(),
            Either::Result(exe_result) => exe_result.to_sexp(),
        }
    }
}
