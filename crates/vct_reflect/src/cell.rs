//! Containers for static storage of type information.
//! 
//! # NonGenericTypeCell
//! 
//! For non generic types, provide the following containers:
//! - [`NonGenericTypeInfoCell`]: Storage [`TypeInfo`]
//! - [`NonGenericTypePathCell`]: Storage [`String`]
//! 
//! Internally, there is an [`OnceLock<T>`], almost no additional expenses.
//! 
//! You can use as follows:
//! 
//! ```ignore
//! # use vct_reflect::cell::NonGenericTypePathCell;
//! # use std::string::ToString;
//! fn type_path() -> &'static str {
//!     static CELL: NonGenericTypePathCell = NonGenericTypePathCell::new();
//!     CELL.get_or_init(||"your_path".to_string())
//! }
//! ```
//! 
//! Of course, if string literal can be used, there is no need to use this container.
//! 
//! 
//! # GenericTypeCell
//! 
//! For non generic types, provide the following containers:
//! - [`GenericTypeInfoCell`]: Storage [`TypeInfo`]
//! - [`GenericTypePathCell`]: Storage [`String`]
//! 
//! If the type is generic, the `static CELL` inside the function may be shared by different types.
//! Therefore, the inner of this container is a [`TypeIdMap<T>`] wrapped in [`RwLock`].
//! 
//! You can use as follows:
//! 
//! ```ignore
//! # use vct_reflect::cell::GenericTypePathCell;
//! # use std::string::ToString;
//! use std::any::type_name;
//! fn type_path<T>() -> &'static str {
//!     static CELL: GenericTypePathCell = GenericTypePathCell::new();
//!     CELL.get_or_insert<T, _>(|| type_name::<T>().to_string())
//! }
//! ```

use crate::info::TypeInfo;
use alloc::{boxed::Box, string::String};
use core::any::{Any, TypeId};
use vct_os::sync::{OnceLock, PoisonError, RwLock};
use vct_utils::collections::TypeIdMap;

mod sealed {
    use super::TypeInfo;
    use alloc::string::String;
    pub trait TypedProperty: 'static {}

    impl TypedProperty for String {}
    impl TypedProperty for TypeInfo {}
}

use sealed::TypedProperty;

/// Container for static storage of non-generic type information
pub struct NonGenericTypeCell<T: TypedProperty>(OnceLock<T>);

/// Container for static storage of non-generic type information
pub type NonGenericTypeInfoCell = NonGenericTypeCell<TypeInfo>;

/// Container for static storage of non-generic type path
///
/// For `&'static str`, there is no need to use this type.
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

// impl<T: TypedProperty> Default for NonGenericTypeCell<T> {
//     #[inline]
//     fn default() -> Self {
//         Self::new()
//     }
// }

/// Container for static storage of type information with generics
///
/// `TypePath::type_path` does not have generics.
///  But if the type itself carries generics, They will share a static CELL.
pub struct GenericTypeCell<T: TypedProperty>(RwLock<TypeIdMap<&'static T>>);

/// Container for static storage of type information with generics
pub type GenericTypeInfoCell = GenericTypeCell<TypeInfo>;

/// Container for static storage of type path with generics
///
/// For `&'static str`, there is no need to use this type.
pub type GenericTypePathCell = GenericTypeCell<String>;

impl<T: TypedProperty> GenericTypeCell<T> {
    #[inline]
    pub const fn new() -> Self {
        Self(RwLock::new(TypeIdMap::new()))
    }

    #[inline(always)]
    pub fn get_or_insert<G, F>(&self, f: F) -> &T
    where
        G: Any + ?Sized,
        F: FnOnce() -> T,
    {
        // Separate to reduce code compilation times
        self.get_or_insert_by_type_id(TypeId::of::<G>(), f)
    }

    // Separate to reduce code compilation times
    #[inline(never)]
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

// impl<T: TypedProperty> Default for GenericTypeCell<T> {
//     #[inline]
//     fn default() -> Self {
//         Self::new()
//     }
// }
