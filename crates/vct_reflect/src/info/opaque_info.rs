use crate::{
    Reflect,
    info::{
        Generics, Type, TypePath, docs_macro::impl_docs_fn, generics::impl_generic_fn,
        type_struct::impl_type_fn,
    },
};

/// Container for storing compile-time type information
///
/// 'Opaque' refers to the inability to see the internal implementation,
/// such as u64, String, and other types.
#[derive(Debug, Clone)]
pub struct OpaqueInfo {
    ty: Type,
    generics: Generics,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl OpaqueInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    // Create a new container
    #[inline]
    pub fn new<T: Reflect + TypePath + ?Sized>() -> Self {
        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }
}
