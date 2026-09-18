use crate::{instruction::Instruction, java_types::StackType};

pub struct Class {
    pub name: String,
    pub methods: Box<[Method]>,
}

pub struct Method {
    pub name: String,
    pub params: Box<[StackType]>,
    pub instructions: Box<[Instruction]>,
}
