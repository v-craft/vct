use alloc::boxed::Box;

use crate::{
    Reflect,
    registry::{TypeRegistry, TypeTraits},
};

pub trait DeserializerProcessor {
    fn try_deserialize<'de, D: serde::Deserializer<'de>>(
        &mut self,
        registration: &TypeTraits,
        registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn Reflect>, D::Error>, D>;
}

impl DeserializerProcessor for () {
    fn try_deserialize<'de, D: serde::Deserializer<'de>>(
        &mut self,
        _registration: &TypeTraits,
        _registry: &TypeRegistry,
        deserializer: D,
    ) -> Result<Result<Box<dyn Reflect>, D::Error>, D> {
        Err(deserializer)
    }
}
