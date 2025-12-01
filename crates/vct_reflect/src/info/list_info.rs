use vct_os::sync::Arc;

use crate::{
    Reflect,
    info::{
        CustomAttributes, Generics, Type, TypeInfo, TypePath, Typed, 
        attributes::{impl_custom_attributes_fn, impl_with_custom_attributes}, 
        docs_macro::impl_docs_fn, generics::impl_generic_fn, type_struct::impl_type_fn,
    },
    ops::List,
};


/// Container for storing compile-time list-like information
#[derive(Clone, Debug)]
pub struct ListInfo {
    ty: Type,
    generics: Generics,
    item_ty: Type,
    item_info: fn() -> &'static TypeInfo,
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl ListInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new container
    #[inline]
    pub fn new<TList: List + TypePath, TItem: Reflect + Typed>() -> Self {
        Self {
            ty: Type::of::<TList>(),
            generics: Generics::new(),
            item_ty: Type::of::<TItem>(),
            item_info: TItem::type_info,
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get the [`TypeInfo`] of list items
    #[inline]
    pub fn item_info(&self) -> &'static TypeInfo {
        (self.item_info)()
    }

    /// Get the [`Type`] of list items
    #[inline]
    pub fn item_ty(&self) -> Type {
        self.item_ty
    }
}
