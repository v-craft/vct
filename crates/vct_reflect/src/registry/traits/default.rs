use crate::{Reflect, info::Typed, registry::FromType};
use alloc::boxed::Box;

/// See [`Default`]
#[derive(Clone)]
pub struct TypeTraitDefault {
    func: fn() -> Box<dyn Reflect>,
}

impl TypeTraitDefault {
    /// Call T's [`Default`]
    ///
    /// [`TypeTraitDefault`] does not have a type flag,
    /// but the functions used internally are type specific.
    #[inline(always)]
    pub fn default(&self) -> Box<dyn Reflect> {
        (self.func)()
    }
}

impl<T: Default + Typed + Reflect> FromType<T> for TypeTraitDefault {
    fn from_type() -> Self {
        Self {
            func: || Box::<T>::default(),
        }
    }
}
