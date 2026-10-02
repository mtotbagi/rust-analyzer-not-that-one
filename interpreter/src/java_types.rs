use std::{collections::HashMap, fmt::Debug};

use crate::abstractions::IntLike;

#[derive(Clone, Copy, Debug)]
pub enum StackValue<T: IntLike> {
    Int(T),
    Float(f32),
    Ref(Option<u32>),
}

impl<T: IntLike> StackValue<T> {
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

pub enum HeapValue<T: IntLike + Clone> {
    Int(T),
    Float(f32),
    Byte(u8),
    Char(u16),
    Short(i16),
    Array {
        ty: SimpleType,
        values: T::Array<HeapValue<T>>,
    },
    Object {
        name: String,
        fields: HashMap<String, HeapValue<T>>,
    },
}

impl<T: IntLike + Clone> Debug for HeapValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(arg0) => f.debug_tuple("Int").field(arg0).finish(),
            Self::Float(arg0) => f.debug_tuple("Float").field(arg0).finish(),
            Self::Byte(arg0) => f.debug_tuple("Byte").field(arg0).finish(),
            Self::Char(arg0) => f.debug_tuple("Char").field(arg0).finish(),
            Self::Short(arg0) => f.debug_tuple("Short").field(arg0).finish(),
            Self::Array { ty, values } => f
                .debug_struct("Array")
                .field("ty", ty)
                .field("values", values)
                .finish(),
            Self::Object { name, fields } => f
                .debug_struct("Object")
                .field("name", name)
                .field("fields", fields)
                .finish(),
        }
    }
}

impl<T: IntLike + Clone> Clone for HeapValue<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Int(arg0) => Self::Int(arg0.clone()),
            Self::Float(arg0) => Self::Float(arg0.clone()),
            Self::Byte(arg0) => Self::Byte(arg0.clone()),
            Self::Char(arg0) => Self::Char(arg0.clone()),
            Self::Short(arg0) => Self::Short(arg0.clone()),
            Self::Array { ty, values } => Self::Array {
                ty: ty.clone(),
                values: values.clone(),
            },
            Self::Object { name, fields } => Self::Object {
                name: name.clone(),
                fields: fields.clone(),
            },
        }
    }
}

impl<T: IntLike> HeapValue<T> {
    pub fn to_stack_value(&self) -> StackValue<T> {
        match self {
            HeapValue::Int(i) => StackValue::Int(*i),
            HeapValue::Float(f) => StackValue::Float(*f),
            HeapValue::Byte(b) => todo!(),
            HeapValue::Short(s) => todo!(),
            HeapValue::Char(c) => StackValue::Int(T::from_i32(*c as i32)),
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
