pub use disc_derive::disc;

pub trait FromDiscriminant<T>: Sized {
    fn from_discriminant(_discriminant: T) -> Option<Self>;
}