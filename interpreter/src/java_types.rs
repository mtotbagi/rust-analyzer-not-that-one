use std::{collections::HashMap, fmt::Debug};

use crate::abstractions::{IntLike, Like};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StackType {
    Int,
    Float,
    Ref,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SimpleType {
    Int,
    Float,
    Ref,
    Boolean,
    Byte,
    Char,
    Short,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum HeapType {
    Class { name: String },
    Array { ty: SimpleType },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Type {
    S(SimpleType),
    H(HeapType),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackValue<I, F, R> {
    Int(I),
    Float(F),
    Ref(R),
}

pub type ConcreteStackVal = StackValue<i32, f32, Option<u32>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimpleValue<I, F, R> {
    Int(I),
    Float(F),
    Ref(R),
    Boolean(I),
    Byte(I),
    Char(I),
    Short(I),
}

pub type ConcreteSimpleVal = SimpleValue<i32, f32, Option<u32>>;

#[derive(Clone, Debug)]
pub enum HeapValue<I: IntLike, R: Like, F: Like> {
    Array {
        ty: SimpleType,
        values: I::Array<SimpleValue<I, R, F>>,
    },
    Object {
        name: String,
        fields: HashMap<String, Value<I, R, F>>,
    },
}

pub type ConcreteHeapVal = HeapValue<i32, f32, Option<u32>>;

#[derive(Clone, Debug)]
pub enum Value<I: IntLike, R: Like, F: Like> {
    S(SimpleValue<I, R, F>),
    H(HeapValue<I, R, F>),
}

pub type ConcreteVal = Value<i32, f32, Option<u32>>;

impl<I, F, R> StackValue<I, F, R> {
    pub fn get_type(&self) -> StackType {
        match self {
            StackValue::Int(_) => StackType::Int,
            StackValue::Float(_) => StackType::Float,
            StackValue::Ref(_) => StackType::Ref,
        }
    }

    pub fn to_simple_value(self) -> SimpleValue<I, F, R> {
        match self {
            StackValue::Int(i) => SimpleValue::Int(i),
            StackValue::Float(f) => SimpleValue::Float(f),
            StackValue::Ref(r) => SimpleValue::Ref(r),
        }
    }
}

impl<I: IntLike, R: Like, F: Like> SimpleValue<I, R, F> {
    pub fn to_stack_value(self) -> StackValue<I, R, F> {
        match self {
            SimpleValue::Int(i) => StackValue::Int(i),
            SimpleValue::Float(f) => StackValue::Float(f),
            SimpleValue::Byte(b) => StackValue::Int(b),
            SimpleValue::Short(s) => StackValue::Int(s),
            SimpleValue::Char(c) => StackValue::Int(c),
            SimpleValue::Ref(r) => StackValue::Ref(r),
            SimpleValue::Boolean(b) => StackValue::Int(b),
        }
    }

    pub fn to_value(self) -> Value<I, R, F> {
        Value::S(self)
    }

    pub fn get_type(&self) -> SimpleType {
        match self {
            SimpleValue::Int(_) => SimpleType::Int,
            SimpleValue::Float(_) => SimpleType::Float,
            SimpleValue::Ref(_) => SimpleType::Ref,
            SimpleValue::Boolean(_) => SimpleType::Boolean,
            SimpleValue::Byte(_) => SimpleType::Byte,
            SimpleValue::Char(_) => SimpleType::Char,
            SimpleValue::Short(_) => SimpleType::Short,
        }
    }
}

impl SimpleType {
    pub fn default_val<I: IntLike, F, R>(&self) -> SimpleValue<I, F, R> {
        match self {
            SimpleType::Int => SimpleValue::Int(I::from_i32(0)),
            SimpleType::Float => todo!(),
            SimpleType::Ref => todo!(),
            SimpleType::Boolean => SimpleValue::Boolean(I::from_i32(0)),
            SimpleType::Byte => SimpleValue::Byte(I::from_i32(0)),
            SimpleType::Char => SimpleValue::Char(I::from_i32(0)),
            SimpleType::Short => SimpleValue::Short(I::from_i32(0)),
        }
    }
}
