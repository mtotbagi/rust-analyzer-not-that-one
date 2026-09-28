use crate::{
    abstractions::IntAbstraction,
    instruction::Instruction,
    java_types::{SimpleRef, SimpleType},
};

#[derive(Clone, Debug)]
pub struct Class<T: IntAbstraction> {
    pub name: String,
    pub methods: Box<[Method<T>]>,
}

impl<T: IntAbstraction> Class<T> {
    pub fn get_method(&self, name: &str) -> Option<&Method<T>> {
        self.methods.iter().find(|m| m.id.name == name)
    }
}

#[derive(Clone, Debug)]
pub struct Method<T: IntAbstraction> {
    pub id: MethodId,
    pub instructions: Box<[Instruction<T>]>,
}

#[derive(Clone, Debug)]
pub struct MethodId {
    pub name: String,
    pub params: Box<[SimpleType]>,
    pub ret_ty: Option<SimpleType>,
}

impl MethodId {
    pub fn to_string(&self) -> String {
        let mut res = self.name.clone();
        res += ":(";
        for param in self.params.iter() {
            res += &MethodId::param_to_string(param);
        }
        res += ")";
        if let Some(ty) = &self.ret_ty {
            res += &MethodId::param_to_string(ty)
        } else {
            res += "V"
        }
        res
    }
    fn param_to_string(ty: &SimpleType) -> String {
        match ty {
            SimpleType::Int => "I".to_string(),
            SimpleType::Float => todo!(),
            SimpleType::Byte => todo!(),
            SimpleType::Char => "C".to_string(),
            SimpleType::Short => todo!(),
            SimpleType::Boolean => "Z".to_string(),
            SimpleType::SimpleRef(r) => match r.as_ref() {
                SimpleRef::Class { name: _ } => todo!(),
                SimpleRef::Array { ty } => "[".to_string() + &MethodId::param_to_string(ty),
            },
        }
    }
}
