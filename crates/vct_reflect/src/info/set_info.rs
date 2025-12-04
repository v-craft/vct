use vct_os::sync::Arc;

use crate::{
    Reflect,
    info::{
        CustomAttributes, Generics, Type, TypeInfo, TypePath, Typed,
        attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
        docs_macro::impl_docs_fn,
        generics::impl_generic_fn,
        type_struct::impl_type_fn,
    },
    ops::Set,
};

/// Container for storing compile-time set-like information
#[derive(Clone, Debug)]
pub struct SetInfo {
    ty: Type,
    generics: Generics,
    value_ty: Type,
    value_info: fn() -> &'static TypeInfo,
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl SetInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new container
    #[inline]
    pub fn new<TSet: Set + TypePath, TValue: Reflect + Typed>() -> Self {
        Self {
            ty: Type::of::<TSet>(),
            generics: Generics::new(),
            value_ty: Type::of::<TValue>(),
            value_info: TValue::type_info,
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get the [`TypeInfo`] of the value
    #[inline]
    pub fn value_info(&self) -> &'static TypeInfo {
        (self.value_info)()
    }

    /// Get the [`Type`] of the value
    #[inline]
    pub fn value_ty(&self) -> Type {
        self.value_ty
    }
}
