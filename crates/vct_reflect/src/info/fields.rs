use alloc::borrow::Cow;
use core::fmt;
use vct_os::sync::Arc;

use crate::info::{
    CustomAttributes, Type, TypeInfo, Typed,
    attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
    docs_macro::impl_docs_fn,
    type_struct::impl_type_fn,
};


/// named field(struct field)
#[derive(Clone, Debug)]
pub struct NamedField {
    ty: Type,
    name: &'static str,
    type_info: fn() -> &'static TypeInfo,
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl NamedField {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new container
    #[inline]
    pub fn new<T: Typed>(name: &'static str) -> Self {
        Self {
            name,
            type_info: T::type_info,
            ty: Type::of::<T>(),
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get field name
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get field type info
    #[inline]
    pub fn type_info(&self) -> &'static TypeInfo {
        (self.type_info)()
    }
}

/// unnamed field(tuple field)
#[derive(Clone, Debug)]
pub struct UnnamedField {
    ty: Type,
    index: usize,
    type_info: fn() -> &'static TypeInfo,
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl UnnamedField {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new container
    #[inline]
    pub fn new<T: Typed>(index: usize) -> Self {
        Self {
            index,
            type_info: T::type_info,
            ty: Type::of::<T>(),
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get field index
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get field type info
    #[inline]
    pub fn type_info(&self) -> &'static TypeInfo {
        (self.type_info)()
    }
}

/// A container for representing field names
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldId {
    Named(Cow<'static, str>),
    Unnamed(usize),
}

impl fmt::Display for FieldId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Named(name) => fmt::Display::fmt(name, f),
            Self::Unnamed(name) => fmt::Display::fmt(name, f),
        }
    }
}
