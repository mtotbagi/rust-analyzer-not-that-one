use crate::{Cond, Op};
use std::fmt::Debug;

pub struct BinOpResult<T: IntLike> {
    pub result: Option<T>,
    pub div_error: bool,
}

pub trait IntLike: ToString + Copy + Debug {
    fn ifz(cond: Cond, value: Self) -> Vec<bool>;
    fn cmp(cond: Cond, lhs: Self, rhs: Self) -> Vec<bool>;
    fn bin_op(op: Op, lhs: Self, rhs: Self) -> BinOpResult<Self>;
    fn neg(self) -> Self;
    fn from_i32(value: i32) -> Self;
}

pub trait IntAbstraction: IntLike {
    fn new_int() -> Self;
    fn new_bool() -> Self;
}

impl IntLike for i32 {
    fn ifz(cond: Cond, value: Self) -> Vec<bool> {
        vec![cond.cmp_with(value as i64, 0)]
    }
    fn from_i32(value: i32) -> Self {
        value
    }

    fn cmp(cond: Cond, lhs: Self, rhs: Self) -> Vec<bool> {
        vec![cond.cmp_with(lhs as i64, rhs as i64)]
    }

    fn bin_op(op: Op, lhs: Self, rhs: Self) -> BinOpResult<Self> {
        match op.op_int(lhs, rhs) {
            Some(result) => BinOpResult {
                result: Some(result),
                div_error: false,
            },
            None => BinOpResult {
                result: None,
                div_error: true,
            },
        }
    }
    
    fn neg(self) -> Self {
        self.wrapping_neg()
    }
}

#[derive(Debug, Clone, Copy)]
enum Sign {
    Neg,
    Zero,
    Pos,
}

impl Sign {
    fn from_usize(i: usize) -> Self {
        match i {
            0 => Self::Neg,
            1 => Self::Zero,
            2 => Self::Pos,
            _ => panic!(),
        }
    }

    fn bin_op(op: Op, lhs: Self, rhs: Self) -> Option<SignSet> {
        match (op, lhs, rhs) {
            (Op::Add | Op::Sub, a, Sign::Zero) => Some(SignSet::from_sign(a)),
            (Op::Add, Sign::Zero, a) => Some(SignSet::from_sign(a)),
            (Op::Sub, Sign::Zero, Sign::Neg) => Some(SignSet::from_sign(Sign::Pos)),
            (Op::Sub, Sign::Zero, Sign::Pos) => Some(SignSet::from_sign(Sign::Neg)),
            // Because over/underflow anything can happen :(
            (Op::Add | Op::Sub, _, _) => Some(SignSet([true,true,true])),
            (Op::Mul, Sign::Zero, _) | (Op::Mul, _, Sign::Zero) => Some(SignSet::from_sign(Sign::Zero)),
            (Op::Mul, _, _) => Some(SignSet([true,true,true])),
            (Op::Div | Op::Rem, _, Sign::Zero) => None,
            (Op::Div | Op::Rem, Sign::Zero, _) => Some(SignSet::from_sign(Sign::Zero)),
            (Op::Div, Sign::Neg, Sign::Pos) => Some(SignSet([true,true,false])),
            (Op::Div, Sign::Pos, Sign::Neg) => Some(SignSet([true,true,false])),
            (Op::Div, Sign::Pos, Sign::Pos) => Some(SignSet([false,true,true])),
            (Op::Div, Sign::Neg, Sign::Neg) => Some(SignSet([true,true,true])),
            (Op::Rem, Sign::Neg, _) => Some(SignSet([true,true,false])),
            (Op::Rem, Sign::Pos, _) => Some(SignSet([false,true,true])),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SignSet([bool; 3]);

impl SignSet {
    fn from_sign(sign: Sign) -> Self {
        match sign {
            Sign::Neg => Self([true, false, false]),
            Sign::Zero => Self([false, true, false]),
            Sign::Pos => Self([false, false, true]),
        }
    }
    fn signs(&self) -> impl Iterator<Item = Sign> {
        self.0.iter().enumerate().filter_map(
            |(i, &b)| {
                if b { Some(Sign::from_usize(i)) } else { None }
            },
        )
    }
    fn union_with(&mut self, other: &Self) {
        for i in 0..3 {
            self.0[i] |= other.0[i]
        }
    }
}

impl IntLike for SignSet {
    fn ifz(cond: Cond, value: Self) -> Vec<bool> {
        let mut res = vec![];
        for sign in value.signs() {
            match (sign, cond) {
                (Sign::Neg, Cond::Eq | Cond::Gt | Cond::Ge | Cond::Is) => res.push(false),
                (Sign::Neg, _) => res.push(true),
                (Sign::Zero, Cond::Eq | Cond::Is | Cond::Le | Cond::Ge) => res.push(true),
                (Sign::Zero, _) => res.push(false),
                (Sign::Pos, Cond::Eq | Cond::Lt | Cond::Le | Cond::Is) => res.push(false),
                (Sign::Pos, _) => res.push(true),
            }
        }

        res
    }

    fn from_i32(value: i32) -> Self {
        if value == 0 {
            Self::from_sign(Sign::Zero)
        } else if value < 0 {
            Self::from_sign(Sign::Neg)
        } else {
            Self::from_sign(Sign::Pos)
        }
    }

    fn cmp(cond: Cond, lhs: Self, rhs: Self) -> Vec<bool> {
        let mut res = vec![];
        for s1 in lhs.signs() {
            for s2 in rhs.signs() {
                match (cond, s1, s2) {
                    (_, Sign::Neg, Sign::Neg) => return vec![true, false],
                    (_, Sign::Pos, Sign::Pos) => return vec![true, false],
                    (Cond::Eq | Cond::Ge | Cond::Le | Cond::Is, Sign::Zero, Sign::Zero) => {
                        res.push(true)
                    }
                    (_, Sign::Zero, Sign::Zero) => res.push(false),
                    (
                        Cond::Ne | Cond::Le | Cond::Lt | Cond::IsNot,
                        Sign::Neg,
                        Sign::Zero | Sign::Pos,
                    ) => res.push(true),
                    (_, Sign::Neg, Sign::Zero | Sign::Pos) => res.push(false),
                    (
                        Cond::Ne | Cond::Ge | Cond::Gt | Cond::IsNot,
                        Sign::Zero | Sign::Pos,
                        Sign::Neg,
                    ) => res.push(true),
                    (_, Sign::Zero | Sign::Pos, Sign::Neg) => res.push(false),
                    (Cond::Ne | Cond::Ge | Cond::Gt | Cond::IsNot, Sign::Pos, Sign::Zero) => {
                        res.push(true)
                    }
                    (_, Sign::Pos, Sign::Zero) => res.push(false),
                    (Cond::Ne | Cond::Le | Cond::Lt | Cond::IsNot, Sign::Zero, Sign::Pos) => {
                        res.push(true)
                    }
                    (_, Sign::Zero, Sign::Pos) => res.push(false),
                }
            }
        }
        res
    }

    fn bin_op(op: Op, lhs: Self, rhs: Self) -> BinOpResult<Self> {
        dbg!(op, lhs, rhs);
        let mut res = Self([true,true,true]);
        let mut div_error = false;
        for s1 in lhs.signs() {
            for s2 in rhs.signs() {
                match Sign::bin_op(op, s1, s2) {
                    Some(a) => res.union_with(&a),
                    None => div_error = true,
                }
            }
        }
        let result = if res.signs().next().is_none() {
            None
        } else {
            Some(res)
        };
        BinOpResult { result, div_error }
    }
    
    fn neg(self) -> Self {
        // Because in wrapping integer arithmetic -int::minvalue = -int::minvalue
        // From neg we can get neg or pos
        Self([self.0[2] | self.0[0], self.0[1], self.0[0]])
    }
}

impl ToString for SignSet {
    fn to_string(&self) -> String {
        self.0.iter().filter(|&&b| b).count().to_string()
    }
}

impl IntAbstraction for SignSet {
    fn new_int() -> Self {
        Self([true, true, true])
    }

    fn new_bool() -> Self {
        Self([false, true, true])
    }
}
