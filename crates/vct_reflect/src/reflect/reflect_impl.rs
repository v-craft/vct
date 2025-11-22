use core::{
    any::{Any, TypeId}, fmt
};
use alloc::boxed::Box;
use crate::{
    PartialReflect,
    cell::NonGenericTypeInfoCell,
    info::{
        DynamicTyped, OpaqueInfo, 
        TypeInfo, TypePath, Typed,
    }
};

pub trait Reflect: PartialReflect + DynamicTyped + Any{
    fn as_reflect(&self) -> &dyn Reflect;
    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;
    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect>;
    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>>;
}

impl dyn Reflect {
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        // Any::Type_id(self)
        self.type_id() == TypeId::of::<T>()
    }

    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        <dyn Any>::downcast_ref(self)
    }

    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        <dyn Any>::downcast_mut(self)
    }

    #[inline]
    pub fn downcast<T: Any>(self: Box<dyn Reflect>) -> Result<Box<T>, Box<dyn Reflect>> {
        if self.is::<T>() {
            // TODO: use downcast_uncheck to reduce once type check
            Ok(<Box<dyn Any>>::downcast(self).unwrap())
        } else {
            Err(self)
        }
    }

    pub fn take<T: Any>(self: Box<dyn Reflect>) -> Result<T, Box<dyn Reflect>> {
        self.downcast::<T>().map(|value| *value)
    }
}

impl fmt::Debug for dyn Reflect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // PartialReflect::debug(self, f)
        self.debug(f)
    }
}

impl TypePath for dyn Reflect {
    fn type_path() -> &'static str {
        "dyn vct_reflect::Reflect"
    }
    fn short_type_path() -> &'static str {
        "dyn Reflect"
    }
}

impl Typed for dyn Reflect {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

// 例：impl_reflect_trait!(<T> for MyStruct<T> where T: 'static)
macro_rules! impl_reflect_trait {
    ($(<$($id:ident),* $(,)?>)? for $ty:ty $(where $($tt:tt)*)?) => {
        impl $(<$($id),*>)? $crate::Reflect for $ty $(where $($tt)*)? {
            #[inline(always)]
            fn as_reflect(&self) -> &dyn $crate::Reflect {
                self
            }

            #[inline(always)]
            fn as_reflect_mut(&mut self) -> &mut dyn $crate::Reflect {
                self
            }

            #[inline(always)]
            fn into_reflect(self: ::alloc::boxed::Box<Self>) -> ::alloc::boxed::Box<dyn $crate::Reflect> {
                self
            }

            #[inline]
            fn set(
                &mut self,
                value: ::alloc::boxed::Box<dyn $crate::Reflect>,
            ) -> Result<(), ::alloc::boxed::Box<dyn $crate::Reflect>> {
                *self = <dyn $crate::Reflect>::take(value)?;
                Ok(())
            }
        }
    };
}

pub(crate) use impl_reflect_trait;
