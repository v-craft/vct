use crate::{
    PartialReflect,
    cell::NonGenericTypeInfoCell,
    info::{DynamicTyped, OpaqueInfo, TypeInfo, TypePath, Typed},
};
use alloc::boxed::Box;
use core::{
    any::{Any, TypeId},
    fmt,
};

pub trait Reflect: PartialReflect + DynamicTyped + Any {
    /// Casts this type to a fully-reflected value.
    fn as_reflect(&self) -> &dyn Reflect;
    // Normal impl: fn (&self)->&dyn Reflect{ self }

    /// Casts this type to a mutable, fully-reflected value.
    fn as_reflect_mut(&mut self) -> &mut dyn Reflect;
    // Normal impl: fn (&mut self)->&mut dyn Reflect{ self }

    /// Casts this type to a boxed, fully-reflected value.
    fn into_reflect(self: Box<Self>) -> Box<dyn Reflect>;
    // Normal impl: fn (self: Box<Self>)->Box<dyn Reflect>{ self }

    /// Performs a type-checked assignment of a reflected value to this value.
    fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>>;
    // Normal impl: See macro `impl_reflect_trait`
}

impl dyn Reflect {
    /// Returns `true` if the underlying value is of type `T`.
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        // Any::Type_id(self)
        self.type_id() == TypeId::of::<T>()
    }

    /// Downcasts the value to type `T` by reference.
    ///
    /// If the underlying value is not of type `T`, returns `None`.
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        <dyn Any>::downcast_ref(self)
    }

    /// Downcasts the value to type `T` by mutable reference.
    ///
    /// If the underlying value is not of type `T`, returns `None`.
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        <dyn Any>::downcast_mut(self)
    }

    /// Downcasts the value to type `T`, consuming the trait object.
    ///
    /// If the underlying value is not of type `T`, returns `Err(self)`.
    #[inline]
    pub fn downcast<T: Any>(self: Box<dyn Reflect>) -> Result<Box<T>, Box<dyn Reflect>> {
        if self.is::<T>() {
            // TODO: Use downcast_uncheck to reduce once type check
            // `Any::downcast_uncheck` is unstable now.
            Ok(<Box<dyn Any>>::downcast(self).unwrap())
        } else {
            Err(self)
        }
    }

    /// Downcasts the value to type `T`, unboxing and consuming the trait object.
    ///
    /// If the underlying value is not of type `T`, returns `Err(self)`.
    #[inline]
    pub fn take<T: Any>(self: Box<dyn Reflect>) -> Result<T, Box<dyn Reflect>> {
        self.downcast::<T>().map(|value| *value)
    }
}

impl fmt::Debug for dyn Reflect {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // PartialReflect::debug(self, f)
        self.reflect_debug(f)
    }
}

impl TypePath for dyn Reflect {
    #[inline]
    fn type_path() -> &'static str {
        "dyn vct_reflect::Reflect"
    }
    #[inline]
    fn type_name() -> &'static str {
        "dyn Reflect"
    }
}

impl Typed for dyn Reflect {
    /// This is the [`TypeInfo`] of [`dyn Reflect`],
    /// not the [`TypeInfo`] of the underlying data!!!!
    ///
    /// Use [`DynamicTyped::reflect_type_info`] to get underlying [`TypeInfo`].
    ///
    /// [`dyn Reflect`]: crate::Reflect
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

// Example: impl_reflect_trait!(<T> for MyStruct<T> where T: 'static)
macro_rules! _impl_reflect_trait {
    ($(<$($id:ident),* $(,)?>)? for $ty:ty $(where $($tt:tt)*)?) => {
        impl $(<$($id),*>)? $crate::Reflect for $ty $(where $($tt)*)? {
            #[inline]
            fn as_reflect(&self) -> &dyn $crate::Reflect {
                self
            }

            #[inline]
            fn as_reflect_mut(&mut self) -> &mut dyn $crate::Reflect {
                self
            }

            #[inline]
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

// pub(crate) use impl_reflect_trait;
