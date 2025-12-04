use serde::Serializer;

use crate::{Reflect, registry::TypeRegistry};

/// A serialization interface where types implementing this trait
/// need to attempt serialization using `&dyn Reflect`, `&TypeRegistry`, and `Serializer`.
///
/// It returns three cases:
/// - `Ok(Ok(_))` indicates successful serialization,
/// - `Ok(Err(_))` indicates serialization failure,
/// - `Err(s)` indicates that the serializer does not support,
///   Therefore, `s`(`serde::Serializer` object) is returned, allowing it to be used in other ways.
pub trait SerializerProcessor {
    fn try_serialize<S: Serializer>(
        &self,
        value: &dyn Reflect,
        registry: &TypeRegistry,
        serializer: S,
    ) -> Result<Result<S::Ok, S::Error>, S>;
}

impl SerializerProcessor for () {
    fn try_serialize<S: Serializer>(
        &self,
        _value: &dyn Reflect,
        _registry: &TypeRegistry,
        serializer: S,
    ) -> Result<Result<S::Ok, S::Error>, S> {
        Err(serializer)
    }
}
