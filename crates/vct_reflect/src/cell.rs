use crate::info::TypeInfo;
use alloc::{boxed::Box, string::String};
use core::any::{Any, TypeId};
use vct_os::sync::{OnceLock, PoisonError, RwLock};
use vct_utils::collections::TypeIdMap;

mod sealed {
    use super::TypeInfo;
    pub trait TypedProperty: 'static {}

    impl TypedProperty for alloc::string::String {}
    impl TypedProperty for TypeInfo {}
}

use sealed::TypedProperty;

/// Container for static storage of non-generic type information
pub struct NonGenericTypeCell<T: TypedProperty>(OnceLock<T>);

/// Container for static storage of non-generic type information
pub type NonGenericTypeInfoCell = NonGenericTypeCell<TypeInfo>;

/// Container for static storage of non-generic type path
pub type NonGenericTypePathCell = NonGenericTypeCell<String>;

impl<T: TypedProperty> NonGenericTypeCell<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(f)
    }
}

impl<T: TypedProperty> Default for NonGenericTypeCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Container for static storage of type information with generics
pub struct GenericTypeCell<T: TypedProperty>(RwLock<TypeIdMap<&'static T>>);

/// Container for static storage of type information with generics
pub type GenericTypeInfoCell = GenericTypeCell<TypeInfo>;

/// Container for static storage of type path with generics
pub type GenericTypePathCell = GenericTypeCell<String>;

impl<T: TypedProperty> GenericTypeCell<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(RwLock::new(TypeIdMap::new()))
    }

    #[inline]
    pub fn get_or_insert<G, F>(&self, f: F) -> &T
    where
        G: Any + ?Sized,
        F: FnOnce() -> T,
    {
        // Separate to reduce code compilation times
        self.get_or_insert_by_type_id(TypeId::of::<G>(), f)
    }

    // Separate to reduce code compilation times
    fn get_or_insert_by_type_id<F>(&self, type_id: TypeId, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get_by_type_id(type_id) {
            Some(info) => info,
            None => self.insert_by_type_id(type_id, f()),
        }
    }

    // Separate to reduce code compilation times
    fn get_by_type_id(&self, type_id: TypeId) -> Option<&T> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&type_id)
            .copied()
    }

    // Separate to reduce code compilation times
    fn insert_by_type_id(&self, type_id: TypeId, value: T) -> &T {
        let mut write_lock = self.0.write().unwrap_or_else(PoisonError::into_inner);

        write_lock
            .entry(type_id)
            .insert({
                // Obtain a reference to the static lifecycle through leak.
                // GenericTypeCell should only be used as a static variable,
                // and the inserted data itself will not be released,
                // Therefore, leak the value has no negative effect.
                Box::leak(Box::new(value))
            })
            .get()
    }
}

impl<T: TypedProperty> Default for GenericTypeCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
