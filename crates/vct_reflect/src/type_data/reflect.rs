use core::any::Any;
use core::fmt::Debug;
use crate::PartialReflect;

pub trait Reflect: PartialReflect /*+ DynamicTyped*/ + Any{
    fn as_any(&self) -> &dyn Any;

    fn debug(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}


impl dyn Reflect {
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

impl Debug for dyn Reflect {
    fn fmt(&self, _f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unimplemented!()
    }
}
