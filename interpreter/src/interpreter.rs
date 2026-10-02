use std::collections::HashMap;

use crate::{
    ExeResult::{NullPointer, OutOfBounds},
    abstractions::{IntAbstraction, IntLike},
    state::ExeResult::{AssertErr, Div0, Ok},
    *,
};
pub struct Interpreter<T: IntLike> {
    class: Class<T>,
}

impl Interpreter<i32> {
    pub fn interpret(
        &self,
        method: &Method<i32>,
        input: (Vec<StackValue<i32>>, Heap<i32>),
        iter: u32,
    ) -> (Vec<State<i32>>, ExeResult) {
        let pc = ProgramCounter {
            class: self.class.name.clone(),
            method: method.id.clone(),
            idx: 0,
        };
        let mut state = State::new(pc.clone(), input.0, input.1);
        let mut states = vec![state.clone()];
        for _ in 0..iter {
            let mut res = self.step(state);
            state = match res.pop().unwrap() {
                Either::State(state) => {
                    states.push(state.clone());
                    state
                }
                Either::Result(exe_result) => return (states, exe_result),
            }
        }
        (states, ExeResult::DidNotFinish)
    }
}

impl<T: IntLike> Interpreter<T> {
    pub fn new(class: Class<T>) -> Self {
        Interpreter { class }
    }

    fn multistep(&self, states: Vec<State<T>>) -> Vec<Either<T>> {
        states
            .into_iter()
            .flat_map(|state| self.step(state))
            .collect()
    }

    fn step(&self, mut state: State<T>) -> Vec<Either<T>> {
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
            Instruction::Throw => return vec![Either::Result(AssertErr)],
            Instruction::Ifz { cond, target } => {
                let poss_outcomes = match cur_frame.stack.pop() {
                    Some(StackValue::Int(i)) => T::ifz(*cond, i),
                    Some(StackValue::Ref(Some(i))) => vec![cond.cmp_with(i as i64, 0)],
                    Some(StackValue::Ref(None)) => vec![cond.cmp_with(0, 0)],
                    Some(_) => panic!(),
                    None => panic!(),
                };
                let mut states = vec![];
                if poss_outcomes.contains(&true) {
                    let mut frame_copy = cur_frame.clone();

                    frame_copy.set_pc(*target);
                    let mut state_copy = state.clone();
                    state_copy.frames.push(frame_copy);
                    states.push(Either::State(state_copy));
                }
                if poss_outcomes.contains(&false) {
                    cur_frame.increment_pc();
                    state.frames.push(cur_frame);
                    states.push(Either::State(state));
                }
                return states;
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
                    return vec![Either::State(state)];
                } else {
                    return vec![Either::Result(Ok)];
                }
            }
            Instruction::Get => {
                // TODO handle other cases than assertionsDisabled = false
                cur_frame.stack.push(StackValue::Int(T::from_i32(0)));
                dbg!(&cur_frame.stack);
            }
            Instruction::New { class } => {
                cur_frame
                    .stack
                    .push(StackValue::Ref(Some(state.heap.heap.len() as u32)));
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
                    SimpleRef::Class { name } => name,
                    SimpleRef::Array { ty: _ } => todo!(),
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
                    return vec![Either::State(state)];
                }
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
                    let op_res = T::bin_op(*op, lhs, rhs);
                    let mut states = vec![];
                    if op_res.div_error {
                        states.push(Either::Result(Div0));
                    }
                    if let Some(value) = op_res.result {
                        cur_frame.stack.push(StackValue::Int(value));
                        cur_frame.increment_pc();
                        state.frames.push(cur_frame);
                        states.push(Either::State(state));
                    }
                    return states;
                }
                StackType::Float => todo!(),
                StackType::Ref => panic!("Cannot use ref for arithmetic operations!"),
            },
            Instruction::If { cond, target } => {
                let rhs = match cur_frame.stack.pop() {
                    Some(StackValue::Int(i)) => i,
                    Some(StackValue::Ref(Some(_))) => todo!(),
                    Some(StackValue::Ref(None)) => todo!(),
                    Some(_) => panic!(),
                    None => panic!(),
                };
                let lhs = match cur_frame.stack.pop() {
                    Some(StackValue::Int(i)) => i,
                    Some(StackValue::Ref(Some(_))) => todo!(),
                    Some(StackValue::Ref(None)) => todo!(),
                    Some(_) => panic!(),
                    None => panic!(),
                };
                let poss_outcomes = T::cmp(*cond, lhs, rhs);
                let mut states = vec![];
                if poss_outcomes.contains(&true) {
                    let mut frame_copy = cur_frame.clone();

                    frame_copy.set_pc(*target);
                    let mut state_copy = state.clone();
                    state_copy.frames.push(frame_copy);
                    states.push(Either::State(state_copy));
                }
                if poss_outcomes.contains(&false) {
                    cur_frame.increment_pc();
                    state.frames.push(cur_frame);
                    states.push(Either::State(state));
                }
                return states;
            }
            Instruction::Goto { target } => {
                cur_frame.set_pc(*target);
                state.frames.push(cur_frame);
                return vec![Either::State(state)];
            }
            Instruction::Placeholder => todo!(),
            Instruction::NewArray { dim, ty } => {
                // length is the product of dimensions
                // array elements are accessed by only one index so dimensions don't need to be saved?
                // let arr_length = (0..dim).map(|| cur_frame.stack.pop()).product();

                if *dim != 1 {
                    todo!("Only one dimensional arrays are supported currently!");
                }

                let len = if let StackValue::Int(x) = cur_frame.stack.pop().unwrap() {
                    x
                } else {
                    panic!("Invalid array length!")
                };

                // 0 as default
                let default_value = match ty {
                    SimpleType::Int => HeapValue::Int(T::from_i32(0)),
                    SimpleType::Float => HeapValue::Float(0.0),
                    SimpleType::Byte => HeapValue::Byte(0),
                    SimpleType::Char => HeapValue::Char(0),
                    SimpleType::Short => HeapValue::Short(0),
                    SimpleType::Boolean => todo!(),
                    SimpleType::SimpleRef(_) => todo!(),
                };
                let arr: <T as IntLike>::Array<HeapValue<T>> = T::new_array(len, default_value);

                cur_frame
                    .stack
                    .push(StackValue::Ref(Some(state.heap.heap.len() as u32)));
                state.heap.heap.push(HeapValue::Array {
                    ty: ty.clone(),
                    values: arr,
                });
            }
            Instruction::ArrayStore { ty } => {
                let val = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: value not on stack");
                let StackValue::Int(idx) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: index not on stack")
                else {
                    panic!("Expected an Int");
                };

                let StackValue::Ref(arr_ref) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: arrayref not on stack")
                else {
                    panic!("Expected a Ref");
                };

                let Some(arr) = arr_ref else {
                    return vec![Either::Result(NullPointer)];
                };

                if let HeapValue::Array { ty, values } = &mut state.heap.heap[arr as usize] {
                    // TODO: check type
                    let res = T::array_store(val.to_heap_value(), idx, values);
                    let mut states = vec![];
                    if res.contains(&true) {
                        cur_frame.increment_pc();
                        state.frames.push(cur_frame);
                        states.push(Either::State(state));
                    }
                    if res.contains(&false) {
                        states.push(Either::Result(OutOfBounds));
                    }
                    return states;
                } else {
                    panic!("Not array ref");
                }
            }
            Instruction::ArrayLength => {
                let StackValue::Ref(arr_ref) = cur_frame
                    .stack
                    .pop()
                    .expect("[ArraySotre]: arrayref not on stack")
                else {
                    panic!("Expected a Ref");
                };

                let Some(arr) = arr_ref else {
                    return vec![Either::Result(NullPointer)];
                };

                if let HeapValue::Array { values, .. } = &state.heap.heap[arr as usize] {
                    cur_frame.push(StackValue::Int(T::array_len(values)));
                } else {
                    panic!("Not array ref");
                }
            }
            // Instruction::ArrayLoad { ty } => {
            //     let StackValue::Int(idx) = cur_frame
            //         .stack
            //         .pop()
            //         .expect("[ArrayLoad]: index not on stack")
            //     else {
            //         panic!("Expected an Int");
            //     };

            //     let StackValue::Ref(arr_ref) = cur_frame
            //         .stack
            //         .pop()
            //         .expect("[ArrayLoad]: arrayref not on stack")
            //     else {
            //         panic!("Expected a Ref");
            //     };

            //     let Some(arr) = arr_ref else {
            //         return Either::Result(NullPointer);
            //     };

            //     if let HeapValue::Array { values, .. } = &state.heap.heap[arr as usize] {
            //         if idx as usize >= values.len() {
            //             return Either::Result(OutOfBounds);
            //         }

            //         cur_frame.push(values[idx as usize].to_stack_value());
            //     } else {
            //         panic!("Not array ref");
            //     }
            // }
            Instruction::Incr { index, amount } => {
                let StackValue::Int(local) = cur_frame.locals[*index as usize].unwrap() else {
                    panic!("Local must be an int, local: {}", index);
                };
                let result = T::bin_op(Op::Add, local, T::from_i32(*amount))
                    .result
                    .unwrap();
                cur_frame.locals[*index as usize] = Some(StackValue::Int(result));
            }
            Instruction::Neg { ty } => {
                let Some(value) = cur_frame.stack.pop() else {
                    panic!()
                };
                assert!(*ty == value.get_type());
                let res = match value {
                    StackValue::Int(v) => StackValue::Int(v.neg()),
                    StackValue::Float(v) => StackValue::Float(-v),
                    StackValue::Ref(_) => panic!(),
                };
                cur_frame.push(res);
            }
            Instruction::NoOp => {}
            _ => todo!(),
        }
        cur_frame.increment_pc();
        state.frames.push(cur_frame);
        vec![Either::State(state)]
    }
}

impl<T: IntAbstraction> Interpreter<T> {
    pub fn abstract_interpret(&self, method: &Method<T>, iter: u32) -> Vec<ExeResult> {
        let input = abstract_input::<T>(&method.id.params);
        let pc = ProgramCounter {
            class: self.class.name.clone(),
            method: method.id.clone(),
            idx: 0,
        };
        let mut states = vec![State::new(pc.clone(), input.0, input.1)];
        let mut results = vec![];
        for _ in 0..iter {
            states = self
                .multistep(states)
                .into_iter()
                .filter_map(|res| match res {
                    Either::State(state) => Some(state),
                    Either::Result(exe_result) => {
                        if !results.contains(&exe_result) {
                            results.push(exe_result);
                        }
                        None
                    }
                })
                .collect();
        }
        if !states.is_empty() {
            results.push(ExeResult::DidNotFinish);
        }
        results
    }
}
