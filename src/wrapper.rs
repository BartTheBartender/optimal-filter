pub trait Wrapper<T: Clone> : Clone {
    type Info;
    fn info(&self) -> Self::Info;
    fn into_object(self) -> T;
    fn from_object(object: T) -> Self;
}

#[derive(Clone)  ]
pub struct TrivialWrapper<T: Clone>(T);

impl<T: Clone> Wrapper<T> for TrivialWrapper<T> {
    type Info = ();

    fn info(&self) -> Self::Info  { }
    fn into_object(self) -> T {self.0}
    fn from_object(object: T) -> Self {Self(object)}
}
