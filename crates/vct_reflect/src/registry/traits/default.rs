use crate::{Reflect, registry::FromType};
use alloc::boxed::Box;

/// See [`Default`]
#[derive(Clone)]
pub struct TypeTraitDefault {
    default: fn() -> Box<dyn Reflect>,
}

impl TypeTraitDefault {
    #[inline(always)]
    pub fn default(&self) -> Box<dyn Reflect> {
        (self.default)()
    }
}

impl<T: Default + Reflect> FromType<T> for TypeTraitDefault {
    fn from_type() -> Self {
        Self {
            default: || Box::<T>::default(),
        }
    }
}
