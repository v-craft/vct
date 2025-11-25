use crate::{
    info::{
        Generics, Type, TypePath, UnnamedField, docs_macro::impl_docs_fn,
        generics::impl_generic_fn, type_struct::impl_type_fn,
    },
    ops::Tuple,
};
use alloc::boxed::Box;

/// Container for storing compile-time tuple information
#[derive(Clone, Debug)]
pub struct TupleInfo {
    ty: Type,
    generics: Generics,
    fields: Box<[UnnamedField]>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl TupleInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    /// Create a new container
    ///
    /// The order of fields inside the container is fixed
    #[inline]
    pub fn new<T: TypePath + Tuple>(fields: &[UnnamedField]) -> Self {
        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            fields: fields.to_vec().into_boxed_slice(),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// Get [`UnnamedField`] by field index
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&UnnamedField> {
        self.fields.get(index)
    }

    /// Get the iter of [`UnnamedField`]
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, UnnamedField> {
        self.fields.iter()
    }

    /// Get the number of fields
    #[inline]
    pub fn field_len(&self) -> usize {
        self.fields.len()
    }
}
