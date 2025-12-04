use crate::{Reflect, info::Typed, registry::FromType};
use serde::Serialize;

/// A struct used to serialize reflected instances of a type.
///
/// This is a fixed type serialization, type errors can cause panic.
#[derive(Clone)]
pub struct TypeTraitSerialize {
    fun: fn(value: &dyn Reflect) -> &dyn erased_serde::Serialize,
}

impl<T: erased_serde::Serialize + Typed + Reflect> FromType<T> for TypeTraitSerialize {
    fn from_type() -> Self {
        Self {
            fun: |value| match value.downcast_ref::<T>() {
                Some(val) => val as &dyn erased_serde::Serialize,
                None => {
                    panic!(
                        "Serial type mismatched, Serial Type `{}` with Value Type: {}",
                        T::type_path(),
                        value.reflect_type_path(),
                    );
                }
            },
        }
    }
}

impl TypeTraitSerialize {
    /// Call T's [`Serialize`]
    ///
    /// [`TypeTraitSerialize`] does not have a type flag,
    /// but the functions used internally are type specific.
    ///
    /// # Panic
    /// - Mismatched Type
    #[inline(always)]
    pub fn serialize<S: serde::Serializer>(
        &self,
        value: &dyn Reflect,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        (self.fun)(value).serialize(serializer)
    }
}
