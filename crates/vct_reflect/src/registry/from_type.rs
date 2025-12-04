use crate::info::Typed;

/// Trait used to generate [`TypeTrait`] for trait reflection.
pub trait FromType<T: Typed> {
    fn from_type() -> Self;
}
