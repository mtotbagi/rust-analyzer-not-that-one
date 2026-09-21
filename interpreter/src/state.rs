use crate::java_types::{HeapValue, SimpleType, StackType, StackValue};

pub enum Either {
    State(State),
    Result(ExeResult),
}

pub enum ExeResult {
    Ok,
    AssertErr,
    OutOfBounds,
    NullPointer,
    Div0,
    NoHalt,
}

pub struct State {
    pub heap: Heap,
    pub frames: Vec<Frame>,
}

impl State {
    pub fn new(program_counter: ProgramCounter, input: Vec<StackValue>, heap: Heap) -> Self {
        Self {
            heap: heap,
            frames: vec![Frame::new(program_counter, input)],
        }
    }
}

pub struct Heap {
    pub heap: Vec<HeapValue>,
}

pub struct Frame {
    pub stack: Vec<StackValue>,
    pub locals: Vec<Option<StackValue>>,
    pub program_counter: ProgramCounter,
}

impl Frame {
    pub fn new(program_counter: ProgramCounter, input: Vec<StackValue>) -> Self {
        Self {
            stack: vec![],
            locals: input.into_iter().map(|v| Some(v)).collect(),
            program_counter,
        }
    }

    pub fn pc(&self) -> usize {
        self.program_counter.1 as usize
    }

    pub fn set_pc(&mut self, target: u32) {
        self.program_counter.1 = target;
    }

    pub fn increment_pc(&mut self) {
        self.program_counter.1 += 1;
    }

    pub fn dup(&mut self, words: u32) {
        if words == 1 {
            self.stack.push(self.stack[self.stack.len() - 1]);
        } else if words == 2 {
            self.stack.push(self.stack[self.stack.len() - 2]);
            self.stack.push(self.stack[self.stack.len() - 2]);
        } else {
            panic!()
        }
    }

    pub(crate) fn push(&mut self, value: StackValue) {
        self.stack.push(value);
    }

    pub(crate) fn load(&mut self, ty: StackType, index: u32) {
        let local = self.locals[index as usize].unwrap();
        assert!(local.get_type() == ty);

        self.stack.push(local);
    }

    pub(crate) fn store(&mut self, ty: StackType, index: u32) {
        let value = self.stack.pop().unwrap();
        assert!(value.get_type() == ty);

        self.locals.resize((index as usize + 1).max(self.locals.len()), None);
        self.locals[index as usize] = Some(value);
    }
}

#[derive(Clone, Debug)]
pub struct ProgramCounter(pub String, pub u32);
