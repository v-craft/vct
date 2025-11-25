use crate::{
    Reflect,
    info::{
        Generics, MaybeTyped, Type, TypeInfo, TypePath, docs_macro::impl_docs_fn,
        generics::impl_generic_fn, type_struct::impl_type_fn,
    },
    ops::Array,
};

/// Container for storing compile-time array information
#[derive(Clone, Debug)]
pub struct ArrayInfo {
    ty: Type,
    generics: Generics,
    item_ty: Type,
    item_info: fn() -> Option<&'static TypeInfo>,
    capacity: usize,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl ArrayInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    /// create a new container
    #[inline]
    pub fn new<TArray: TypePath + Array, TItem: MaybeTyped + TypePath + Reflect>(
        capacity: usize,
    ) -> Self {
        Self {
            ty: Type::of::<TArray>(),
            generics: Generics::new(),
            item_ty: Type::of::<TItem>(),
            item_info: TItem::maybe_type_info,
            capacity,
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
    pub fn item_info(&self) -> Option<&'static TypeInfo> {
        (self.item_info)()
    }

    /// Get the `Type` of array item
    #[inline]
    pub fn item_ty(&self) -> Type {
        self.item_ty
    }
}
