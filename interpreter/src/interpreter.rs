use std::collections::HashMap;

use crate::{
    state::ExeResult::{AssertErr, Div0, NullPointer, Ok, OutOfBounds},
    *,
};
pub struct Interpreter {
    class: Class,
}
impl Interpreter {
    pub fn new(class: Class) -> Self {
        Interpreter { class }
    }

    pub fn interpret(
        &self,
        method: &Method,
        input: (Vec<ConcreteStackVal>, ConcreteHeap),
        iter: u32,
    ) -> (Vec<ConcreteState>, ExeResult) {
        let pc = ProgramCounter {
            class: self.class.name.clone(),
            method: method.id.clone(),
            idx: 0,
        };
        let mut state = ConcreteState::new(pc.clone(), input.0, input.1);
        let mut states = vec![state.clone()];
        for _ in 0..iter {
            let res = self.step(state);
            state = match res {
                Either::Left(state) => {
                    states.push(state.clone());
                    state
                }
                Either::Right(exe_result) => return (states, exe_result),
            }
        }
        (states, ExeResult::DidNotFinish)
    }

    fn step(&self, mut state: ConcreteState) -> Either<ConcreteState, ExeResult> {
        let Some(mut cur_frame) = state.frames.pop() else {
            panic!("Empty state!")
        };
        let bytecode = self
            .class
            .methods
            .iter()
            .filter(|m| m.id.name == cur_frame.program_counter.method.name)
            .map(|m| &m.instructions)
            .next()
            .unwrap();
        dbg!(&bytecode[cur_frame.pc()]);
        match &bytecode[cur_frame.pc()] {
            Instruction::Load { ty, index } => cur_frame.load(*ty, *index),
            Instruction::Push { value } => cur_frame.push(*value),
            Instruction::Dup { words } => cur_frame.dup(*words),
            Instruction::Throw => return Either::Right(AssertErr),
            Instruction::Ifz { cond, target } => {
                let i = match cur_frame.stack.pop() {
                    Some(ConcreteStackVal::Int(i)) => i as i64,
                    Some(ConcreteStackVal::Ref(Some(i))) => i as i64,
                    Some(ConcreteStackVal::Ref(None)) => 0,
                    Some(_) => panic!(),
                    None => panic!(),
                };
                if cond.cmp_with(i, 0) {
                    cur_frame.set_pc(*target);
                    state.frames.push(cur_frame);
                    return Either::Left(state);
                }
            }
            Instruction::Store { ty, index } => cur_frame.store(*ty, *index),
            Instruction::Return { ty } => {
                if let Some(old_frame) = state.frames.last_mut() {
                    if let Some(ty) = ty {
                        let Some(value) = cur_frame.stack.pop() else {
                            panic!(
                                "Invalid frame {}, stack shouldn't be empty!",
                                cur_frame.to_sexp()
                            )
                        };
                        assert!(*ty == value.get_type());
                        old_frame.push(value);
                    }
                    return Either::Left(state);
                } else {
                    return Either::Right(Ok);
                }
            }
            Instruction::Get => {
                // TODO handle other cases than assertionsDisabled = false
                cur_frame.stack.push(ConcreteStackVal::Int(0));
                dbg!(&cur_frame.stack);
            }
            Instruction::New { class } => {
                cur_frame
                    .stack
                    .push(ConcreteStackVal::Ref(Some(state.heap.heap.len() as u32)));
                state.heap.heap.push(HeapValue::Object {
                    name: class.clone(),
                    fields: HashMap::new(),
                });
            }
            Instruction::Invoke {
                access: _,
                method_id,
                simple_ref,
            } => {
                eprintln!("{}", method_id.name);
                // If this is not a method of the class, it's one of the special cases
                // Or it's unhandled and we panic
                let classname = match simple_ref {
                    HeapType::Class { name } => name,
                    HeapType::Array { ty: _ } => todo!(),
                };
                if self.class.name != *classname {
                    if classname == "java/lang/AssertionError" && method_id.name == "<init>" {
                    } else {
                        todo!(
                            "Handling {} method on class {} not yet implemented",
                            method_id.name,
                            classname
                        );
                    }
                } else {
                    let new_pc = ProgramCounter {
                        class: self.class.name.clone(),
                        method: method_id.clone(),
                        idx: 0,
                    };
                    let new_locals = cur_frame
                        .stack
                        .drain(
                            cur_frame.stack.len() - method_id.params.len()..cur_frame.stack.len(),
                        )
                        .collect();
                    let new_frame = Frame::new(new_pc, new_locals);
                    cur_frame.increment_pc();
                    state.frames.push(cur_frame);
                    state.frames.push(new_frame);
                    return Either::Left(state);
                }
            }
            Instruction::Binary { op, ty } => match ty {
                StackType::Int => {
                    let Some(ConcreteStackVal::Int(rhs)) = cur_frame.stack.pop() else {
                        panic!()
                    };
                    let Some(ConcreteStackVal::Int(lhs)) = cur_frame.stack.pop() else {
                        panic!()
                    };
                    dbg!(lhs, rhs);
                    match op.op_int(lhs, rhs) {
                        Some(result) => cur_frame.stack.push(ConcreteStackVal::Int(result)),
                        None => return Either::Right(Div0),
                    }
                }
                StackType::Float => todo!(),
                StackType::Ref => panic!("Cannot use ref for arithmetic operations!"),
            },
            Instruction::If { cond, target } => {
                let rhs = match cur_frame.stack.pop() {
                    Some(ConcreteStackVal::Int(i)) => i as i64,
                    Some(ConcreteStackVal::Ref(Some(i))) => i as i64,
                    Some(ConcreteStackVal::Ref(None)) => -1,
                    Some(_) => panic!(),
                    None => panic!(),
                };
                let lhs = match cur_frame.stack.pop() {
                    Some(ConcreteStackVal::Int(i)) => i as i64,
                    Some(ConcreteStackVal::Ref(Some(i))) => i as i64,
                    Some(ConcreteStackVal::Ref(None)) => -1,
                    Some(_) => panic!(),
                    None => panic!(),
                };

                if cond.cmp_with(lhs, rhs) {
                    cur_frame.set_pc(*target);
                    state.frames.push(cur_frame);
                    return Either::Left(state);
                }
            }
            Instruction::Goto { target } => {
                cur_frame.set_pc(*target);
                state.frames.push(cur_frame);
                return Either::Left(state);
            }
            Instruction::Placeholder => todo!(),
            Instruction::NewArray { dim, ty } => {
                // length is the product of dimensions
                // array elements are accessed by only one index so dimensions don't need to be saved?
                // let arr_length = (0..dim).map(|| cur_frame.stack.pop()).product();

                if *dim != 1 {
                    todo!("Only one dimensional arrays are supported currently!");
                }

                let len = if let ConcreteStackVal::Int(x) = cur_frame.stack.pop().unwrap() {
                    x as usize
                } else {
                    panic!("Invalid array length!")
                };

                // 0 as default
                let arr = vec![ty.default_val(); len];

                cur_frame
                    .stack
                    .push(ConcreteStackVal::Ref(Some(state.heap.heap.len() as u32)));
                state.heap.heap.push(HeapValue::Array {
                    ty: ty.clone(),
                    values: arr,
                });
            }
            Instruction::ArrayStore { ty: _ } => {
                let val = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: value not on stack");
                let ConcreteStackVal::Int(idx) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: index not on stack")
                else {
                    panic!("Expected an Int");
                };

                let ConcreteStackVal::Ref(arr_ref) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: arrayref not on stack")
                else {
                    panic!("Expected a Ref");
                };

                let Some(arr) = arr_ref else {
                    return Either::Right(NullPointer);
                };

                if let HeapValue::Array { ty: _, values } = &mut state.heap.heap[arr as usize] {
                    // TODO: check type
                    if idx as usize >= values.len() {
                        return Either::Right(OutOfBounds);
                    }

                    values[idx as usize] = val.to_simple_value();
                } else {
                    panic!("Not array ref");
                }
            }
            Instruction::ArrayLength => {
                let ConcreteStackVal::Ref(arr_ref) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: arrayref not on stack")
                else {
                    panic!("Expected a Ref");
                };

                let Some(arr) = arr_ref else {
                    return Either::Right(NullPointer);
                };

                if let HeapValue::Array { values, .. } = &state.heap.heap[arr as usize] {
                    cur_frame.push(ConcreteStackVal::Int(values.len() as i32));
                } else {
                    panic!("Not array ref");
                }
            }
            Instruction::ArrayLoad { ty: _ } => {
                let ConcreteStackVal::Int(idx) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArrayLoad]: index not on stack")
                else {
                    panic!("Expected an Int");
                };

                let ConcreteStackVal::Ref(arr_ref) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArrayLoad]: arrayref not on stack")
                else {
                    panic!("Expected a Ref");
                };

                let Some(arr) = arr_ref else {
                    return Either::Right(NullPointer);
                };

                if let HeapValue::Array { values, .. } = &state.heap.heap[arr as usize] {
                    if idx as usize >= values.len() {
                        return Either::Right(OutOfBounds);
                    }

                    cur_frame.push(values[idx as usize].to_stack_value());
                } else {
                    panic!("Not array ref");
                }
            }
            Instruction::Incr { index, amount } => {
                let ConcreteStackVal::Int(local) = cur_frame.locals[*index as usize].unwrap()
                else {
                    panic!("Local must be an int, local: {}", index);
                };
                cur_frame.locals[*index as usize] = Some(ConcreteStackVal::Int(local + *amount));
            }
            Instruction::Neg { ty } => {
                let Some(value) = cur_frame.stack.pop() else {
                    panic!()
                };
                assert!(*ty == value.get_type());
                let res = match value {
                    ConcreteStackVal::Int(v) => ConcreteStackVal::Int(-v),
                    ConcreteStackVal::Float(v) => ConcreteStackVal::Float(-v),
                    ConcreteStackVal::Ref(_) => panic!(),
                };
                cur_frame.push(res);
            }
            Instruction::NoOp => {}
        }
        cur_frame.increment_pc();
        state.frames.push(cur_frame);
        Either::Left(state)
    }
}
