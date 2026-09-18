use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum StackValue {
    Int(i32),
    Float(f32),
    Ref(u32),
}

impl StackValue {
    pub fn get_type(&self) -> StackType {
        match self {
            StackValue::Int(_) => StackType::Int,
            StackValue::Float(_) => StackType::Float,
            StackValue::Ref(_) => StackType::Ref,
        }
    }
}

#[derive(Clone, Debug)]
pub enum HeapValue {
    Int(i32),
    Float(i32),
    Byte(u8),
    Char(u16),
    Short(i16),
    Array {
        ty: SimpleType,
        values: Vec<HeapValue>,
    },
    Object {
        name: String,
        fields: HashMap<String, HeapValue>,
    },
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
