use std::{
    cmp::{Ord, PartialOrd},
    fmt,
};

#[derive(PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Cell<T: Clone + Sized + Ord>(pub T, pub usize);

impl<T: Clone + Sized + Ord> Cell<T> {
    pub const fn name(&self) -> &T {
        &self.0
    }
    pub const fn dim(&self) -> usize {
        self.1
    }
}

impl<T: Clone + fmt::Display + Sized + Ord> fmt::Debug for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cell({}, {})", self.0, self.1)
    }
}

impl<T: Clone + fmt::Display + Sized + Ord> fmt::Display for Cell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}
