use core::{
    fmt,
    any::Any
};
use crate::type_data::PartialReflect;

pub trait Reflect: PartialReflect /*+ DynamicTyped*/ + Any{
    fn as_any(&self) -> &dyn Any;

    fn debug(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}


impl dyn Reflect {
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

impl fmt::Debug for dyn Reflect {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        unimplemented!()
    }
}

