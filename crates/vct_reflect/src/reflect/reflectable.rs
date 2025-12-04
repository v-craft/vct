use crate::{
    Reflect,
    info::{TypePath, Typed},
    registry::GetTypeTraits,
};

// A trait used to simplify constraints
pub trait Reflectable: Reflect + Typed + TypePath + GetTypeTraits {}

impl<T: Reflect + Typed + TypePath + GetTypeTraits> Reflectable for T {}

// [`FromReflect`] is unnecessary, Although the `#[derive(Reflect)]` macro may be impl it
