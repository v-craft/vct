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
    ops::Array,
};

/// Container for storing compile-time array information
#[derive(Clone, Debug)]
pub struct ArrayInfo {
    ty: Type,
    generics: Generics,
    item_ty: Type,
    item_info: fn() -> &'static TypeInfo, // `TypeInfo` is created on the first visit
    capacity: usize,
    custom_attributes: Option<Arc<CustomAttributes>>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl ArrayInfo {
    impl_type_fn!(ty);
    impl_docs_fn!(docs);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// create a new container
    ///
    /// During tuple implementation, there may be a large number of generic expansions.
    /// So inlining is prohibited here.
    #[inline(never)]
    pub fn new<TArray: Array + TypePath, TItem: Reflect + Typed>(capacity: usize) -> Self {
        Self {
            ty: Type::of::<TArray>(),
            generics: Generics::new(),
            item_ty: Type::of::<TItem>(),
            item_info: TItem::type_info,
            capacity,
            custom_attributes: None,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get array capacity (fixed)
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the `TypeInfo` of array items
    #[inline]
    pub fn item_info(&self) -> &'static TypeInfo {
        (self.item_info)()
    }

    /// Get the `Type` of array item
    #[inline]
    pub fn item_ty(&self) -> Type {
        self.item_ty
    }
}
