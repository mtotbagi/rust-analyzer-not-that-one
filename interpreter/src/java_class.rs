use crate::{instruction::Instruction, java_types::SimpleType};

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: Box<[Method]>,
}

#[derive(Clone, Debug)]
pub struct Method {
    pub id: MethodId,
    pub instructions: Box<[Instruction]>,
}

#[derive(Clone, Debug)]
pub struct MethodId {
    pub name: String,
    pub params: Box<[SimpleType]>,
}
