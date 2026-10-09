use crate::{HeapType, SimpleType, instruction::Instruction, java_types::Type};

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: Box<[Method]>,
}

impl Class {
    pub fn get_method(&self, name: &str) -> Option<&Method> {
        self.methods.iter().find(|m| m.id.name == name)
    }
}

#[derive(Clone, Debug)]
pub struct Method {
    pub id: MethodId,
    pub instructions: Box<[Instruction]>,
}

#[derive(Clone, Debug)]
pub struct MethodId {
    pub name: String,
    pub params: Box<[Type]>,
    pub ret_ty: Option<Type>,
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
    fn param_to_string(ty: &Type) -> String {
        match ty {
            Type::S(s) => MethodId::arrkind_to_string(*s),
            Type::H(HeapType::Array { ty }) => "[".to_string() + &MethodId::arrkind_to_string(*ty),
            Type::H(HeapType::Class { name }) => "L".to_string() + name + ";",
        }
    }

    fn arrkind_to_string(kind: SimpleType) -> String {
        match kind {
            SimpleType::Int => "I".to_string(),
            SimpleType::Float => todo!(),
            SimpleType::Ref => todo!(),
            SimpleType::Boolean => "Z".to_string(),
            SimpleType::Byte => todo!(),
            SimpleType::Char => "C".to_string(),
            SimpleType::Short => todo!(),
        }
    }
}
