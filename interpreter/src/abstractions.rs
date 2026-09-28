use crate::Cond;
use std::fmt::Debug;

pub trait IntAbstraction: ToString + Copy + Debug {
    fn ifz(cond: Cond, value: Self) -> Vec<bool>;
    fn cmp(cond: Cond, lhs: Self, rhs: Self) -> Vec<bool>;
    fn from_i32(value: i32) -> Self;
}

impl IntAbstraction for i32 {
    fn ifz(cond: Cond, value: Self) -> Vec<bool> {
        vec![cond.cmp_with(value as i64, 0)]
    }
    fn from_i32(value: i32) -> Self {
        value
    }

    fn cmp(cond: Cond, lhs: Self, rhs: Self) -> Vec<bool> {
        vec![cond.cmp_with(lhs as i64, rhs as i64)]
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
}

#[derive(Debug, Clone, Copy)]
struct SignSet([bool; 3]);

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
}

impl IntAbstraction for SignSet {
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
}

impl ToString for SignSet {
    fn to_string(&self) -> String {
        self.0.iter().filter(|&&b| b).count().to_string()
    }
}
