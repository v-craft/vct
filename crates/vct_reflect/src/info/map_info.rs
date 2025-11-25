use crate::{
    Reflect,
    info::{
        Generics, MaybeTyped, Type, TypeInfo, TypePath, docs_macro::impl_docs_fn,
        generics::impl_generic_fn, type_struct::impl_type_fn,
    },
    ops::Map,
};

/// Container for storing compile-time map-like information
#[derive(Clone, Debug)]
pub struct MapInfo {
    ty: Type,
    generics: Generics,
    key_ty: Type,
    value_ty: Type,
    key_info: fn() -> Option<&'static TypeInfo>,
    value_info: fn() -> Option<&'static TypeInfo>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl MapInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    /// Create a new container
    #[inline]
    pub fn new<
        TMap: TypePath + Map,
        TKey: MaybeTyped + TypePath + Reflect,
        TValue: MaybeTyped + TypePath + Reflect,
    >() -> Self {
        Self {
            ty: Type::of::<TMap>(),
            generics: Generics::new(),
            key_ty: Type::of::<TKey>(),
            value_ty: Type::of::<TValue>(),
            key_info: TKey::maybe_type_info,
            value_info: TValue::maybe_type_info,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get the [`TypeInfo`] of the key
    #[inline]
    pub fn key_info(&self) -> Option<&'static TypeInfo> {
        (self.key_info)()
    }

    /// Get the [`Type`] of the key
    #[inline]
    pub fn key_ty(&self) -> Type {
        self.key_ty
    }

    /// Get the [`TypeInfo`] of the value
    #[inline]
    pub fn value_info(&self) -> Option<&'static TypeInfo> {
        (self.value_info)()
    }

    /// Get the [`Type`] of the value
    #[inline]
    pub fn value_ty(&self) -> Type {
        self.value_ty
    }
}
