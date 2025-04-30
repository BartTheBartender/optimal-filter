use std::fmt::Debug;
pub trait Wrapper<T: Clone + Send + Sync + Debug>: Clone + Send + Sync + Debug {
    type Info: Eq;
    fn info(&self) -> Self::Info;
    fn into_object(self) -> T;
    fn from_object(object: T) -> Self;
}

#[derive(Clone, Debug)]
pub struct TrivialWrapper<T: Clone + Send + Sync + Debug>(T);

impl<T: Clone + Send + Sync + Debug> Wrapper<T> for TrivialWrapper<T> {
    type Info = ();

    fn info(&self) -> Self::Info {}
    fn into_object(self) -> T {
        self.0
    }
    fn from_object(object: T) -> Self {
        Self(object)
    }
}
