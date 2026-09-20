use crate::{
    java_class::MethodId,
    java_types::{SimpleRef, StackType, StackValue},
};

#[derive(Clone, Debug)]
pub enum Instruction {
    Ifz {
        cond: Cond,
        target: u32,
    },
    Load {
        ty: StackType,
        index: u32,
    },
    Store {
        ty: StackType,
        index: u32,
    },
    Push {
        value: StackValue,
    },
    Return {
        ty: Option<StackType>,
    },
    Get,
    New {
        class: String,
    },
    Dup {
        words: u32,
    },
    Invoke {
        access: Access,
        method_id: MethodId,
        simple_ref: SimpleRef,
    },
    Throw,
    Binary {
        op: Op,
        ty: StackType,
    },
    If {
        cond: Cond,
        target: u32,
    },
    Goto {
        target: u32,
    },
    Placeholder,
}

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl Op {
    pub fn op_int(&self, lhs: i32, rhs: i32) -> Option<i32> {
        match self {
            Op::Add => Some(lhs.wrapping_add(rhs)),
            Op::Sub => Some(lhs.wrapping_sub(rhs)),
            Op::Mul => Some(lhs.wrapping_mul(rhs)),
            Op::Div => lhs.checked_div(rhs),
            Op::Rem => Some(lhs.wrapping_rem(rhs)),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Access {
    Special,
    Virtual,
    Static,
    Interface,
    Dynamic,
}

#[derive(Clone, Debug)]
pub enum Cond {
    Ne,
    Eq,
    Lt,
    Le,
    Gt,
    Ge,
    Is,
    IsNot,
}

impl Cond {
    pub fn cmp_with(&self, lhs: i64, rhs: i64) -> bool {
        match self {
            Cond::Ne => lhs != rhs,
            Cond::Eq => lhs == rhs,
            Cond::Lt => lhs < rhs,
            Cond::Le => lhs <= rhs,
            Cond::Gt => lhs > rhs,
            Cond::Ge => lhs >= rhs,
            Cond::Is => lhs == rhs,
            Cond::IsNot => lhs != rhs,
        }
    }
}
