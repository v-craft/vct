use std::boxed::Box;

use crate::{Reflect, info::Typed, registry::FromType};
use serde::Deserialize;

/// A struct used to deserialize date to a reflected instances of a type.
#[derive(Clone)]
pub struct TypeTraitDeserialize {
    func: fn(
        deserializer: &mut dyn erased_serde::Deserializer,
    ) -> Result<Box<dyn Reflect>, erased_serde::Error>,
}

impl TypeTraitDeserialize {
    /// Deserializes a reflected value.
    ///
    /// The underlying type of the reflected value, and thus the expected
    /// structure of the serialized data, is determined by the type used to
    /// construct this `ReflectDeserialize` value.
    #[inline(always)]
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        &self,
        deserializer: D,
    ) -> Result<Box<dyn Reflect>, D::Error> {
        let mut erased = <dyn erased_serde::Deserializer>::erase(deserializer);
        (self.func)(&mut erased).map_err(<D::Error as serde::de::Error>::custom)
    }
}

impl<T: for<'a> Deserialize<'a> + Typed + Reflect> FromType<T> for TypeTraitDeserialize {
    fn from_type() -> Self {
        Self {
            func: |deserializer| Ok(Box::new(T::deserialize(deserializer)?)),
        }
    }
}
