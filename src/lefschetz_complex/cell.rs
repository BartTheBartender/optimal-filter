use std::{
    cmp::{Ord, PartialOrd},
    fmt,
};

#[derive(PartialEq, Eq, Copy, Clone, PartialOrd, Ord)]
pub struct Cell<T: PartialEq + Eq + Copy + Clone + Sized + PartialOrd + Ord>(pub T, pub usize);

impl<T: PartialEq + Eq + Copy + Clone + Sized + PartialOrd + Ord> Cell<T> {
    pub const fn name(&self) -> T {
        self.0
    }
    pub const fn dim(&self) -> usize {
        self.1
    }
}

impl<T: PartialEq + Eq + Copy + Clone + fmt::Debug + Sized + PartialOrd + Ord> fmt::Debug
    for Cell<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cell({:?}, {})", self.0, self.1)
    }
}

impl<T: PartialEq + Eq + Copy + Clone + fmt::Debug + Sized + PartialOrd + Ord> fmt::Display
    for Cell<T>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
