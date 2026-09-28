use std::collections::HashMap;

use crate::abstractions::IntAbstraction;

#[derive(Clone, Copy, Debug)]
pub enum StackValue<T: IntAbstraction> {
    Int(T),
    Float(f32),
    Ref(Option<u32>),
}

impl<T: IntAbstraction> StackValue<T> {
    pub fn get_type(&self) -> StackType {
        match self {
            StackValue::Int(_) => StackType::Int,
            StackValue::Float(_) => StackType::Float,
            StackValue::Ref(_) => StackType::Ref,
        }
    }

    pub fn to_heap_value(&self) -> HeapValue<T> {
        match self {
            StackValue::Int(i) => HeapValue::Int(*i),
            StackValue::Float(f) => HeapValue::Float(*f),
            StackValue::Ref(_) => panic!("Ref can't be turned into a heap value"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum HeapValue<T: IntAbstraction> {
    Int(T),
    Float(f32),
    Byte(u8),
    Char(u16),
    Short(i16),
    Array {
        ty: SimpleType,
        values: Vec<HeapValue<T>>,
    },
    Object {
        name: String,
        fields: HashMap<String, HeapValue<T>>,
    },
}

impl<T: IntAbstraction> HeapValue<T> {
    pub fn to_stack_value(&self) -> StackValue<T> {
        match self {
            HeapValue::Int(i) => StackValue::Int(*i),
            HeapValue::Float(f) => StackValue::Float(*f),
            HeapValue::Byte(b) => todo!(),
            HeapValue::Short(s) => todo!(),
            HeapValue::Char(c) => todo!(),
            _ => panic!("Can't convert to StackValue"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SimpleRef {
    Class { name: String },
    Array { ty: SimpleType },
}

#[derive(Clone, Debug)]
pub enum SimpleType {
    Int,
    // Long,
    Float,
    // Double,
    Byte,
    Char,
    Short,
    Boolean,
    SimpleRef(Box<SimpleRef>),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackType {
    Int,
    Float,
    Ref,
}
