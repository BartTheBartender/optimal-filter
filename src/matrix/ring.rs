use std::fmt;
use std::ops::{Add, BitXor};

pub trait Ring: Add<Output = Self> + Sized + Clone + Copy + PartialEq + Eq + From<i32> {
    const ZERO: Self;
    const ONE: Self;
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Z2(pub bool);

impl Z2 {
    pub const fn new(value: bool) -> Self {
        Self(value)
    }
}

impl Add for Z2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(BitXor::bitxor(self.0, rhs.0))
    }
}

impl Ring for Z2 {
    const ZERO: Self = Self(false);

    const ONE: Self = Self(true);
}

impl From<i32> for Z2 {
    fn from(value: i32) -> Self {
        Self((value & 1) == 1)
    }
}

impl fmt::Debug for Z2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 {
            write!(f, "1")
        } else {
            write!(f, " ")
        }
    }
}
impl fmt::Display for Z2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
