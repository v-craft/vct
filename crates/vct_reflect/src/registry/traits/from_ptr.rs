use core::any::TypeId;
use vct_ptr::{Ptr, PtrMut};

use crate::{Reflect, info::Typed, registry::FromType};

#[derive(Clone)]
pub struct TypeTraitFromPtr {
    type_id: TypeId,
    from_ptr: unsafe fn(Ptr) -> &dyn Reflect,
    from_ptr_mut: unsafe fn(PtrMut) -> &mut dyn Reflect,
}

#[expect(unsafe_code, reason = "Cast pointers to references is unsafe.")]
impl TypeTraitFromPtr {
    /// Returns the [`TypeId`] that the [`ReflectFromPtr`] was constructed for.
    pub fn type_id(&self) -> TypeId {
        self.type_id
    }

    /// Convert `Ptr` into `&dyn Reflect`.
    ///
    /// # Safety
    ///
    /// `val` must be a pointer to value of the type that the [`ReflectFromPtr`] was constructed for.
    /// This can be verified by checking that the type id returned by [`ReflectFromPtr::type_id`] is the expected one.
    pub unsafe fn as_reflect<'a>(&self, val: Ptr<'a>) -> &'a dyn Reflect {
        // SAFETY: contract uphold by the caller.
        unsafe { (self.from_ptr)(val) }
    }

    /// Convert `PtrMut` into `&mut dyn Reflect`.
    ///
    /// # Safety
    ///
    /// `val` must be a pointer to a value of the type that the [`ReflectFromPtr`] was constructed for
    /// This can be verified by checking that the type id returned by [`ReflectFromPtr::type_id`] is the expected one.
    pub unsafe fn as_reflect_mut<'a>(&self, val: PtrMut<'a>) -> &'a mut dyn Reflect {
        // SAFETY: contract uphold by the caller.
        unsafe { (self.from_ptr_mut)(val) }
    }

    /// Get a function pointer to turn a `Ptr` into `&dyn Reflect` for
    /// the type this [`ReflectFromPtr`] was constructed for.
    ///
    /// # Safety
    ///
    /// When calling the unsafe function returned by this method you must ensure that:
    /// - The input `Ptr` points to the `Reflect` type this `ReflectFromPtr`
    ///   was constructed for.
    pub fn from_ptr(&self) -> unsafe fn(Ptr) -> &dyn Reflect {
        self.from_ptr
    }

    /// Get a function pointer to turn a `PtrMut` into `&mut dyn Reflect` for
    /// the type this [`ReflectFromPtr`] was constructed for.
    ///
    /// # Safety
    ///
    /// When calling the unsafe function returned by this method you must ensure that:
    /// - The input `PtrMut` points to the `Reflect` type this `ReflectFromPtr`
    ///   was constructed for.
    pub fn from_ptr_mut(&self) -> unsafe fn(PtrMut) -> &mut dyn Reflect {
        self.from_ptr_mut
    }
}

#[expect(unsafe_code, reason = "Cast pointers to references is unsafe.")]
impl<T: Typed + Reflect> FromType<T> for TypeTraitFromPtr {
    fn from_type() -> Self {
        TypeTraitFromPtr {
            type_id: TypeId::of::<T>(),
            from_ptr: |ptr| {
                // SAFETY: `from_ptr_mut` is either called in `ReflectFromPtr::as_reflect`
                // or returned by `ReflectFromPtr::from_ptr`, both lay out the invariants
                // required by `deref`
                unsafe { ptr.deref::<T>() as &dyn Reflect }
            },
            from_ptr_mut: |ptr| {
                // SAFETY: same as above, but for `as_reflect_mut`, `from_ptr_mut` and `deref_mut`.
                unsafe { ptr.deref_mut::<T>() as &mut dyn Reflect }
            },
        }
    }
}
