use crate::{
    info::{
        CustomAttributes, Generics, Type, TypePath, UnnamedField,
        attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
        docs_macro::impl_docs_fn,
        generics::impl_generic_fn,
        type_struct::impl_type_fn,
    },
    ops::TupleStruct,
};
use alloc::boxed::Box;
use vct_os::sync::Arc;

/// Container for storing compile-time tuple_struct information
#[derive(Clone, Debug)]
pub struct TupleStructInfo {
    ty: Type,
    generics: Generics,
    fields: Box<[UnnamedField]>,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl TupleStructInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new container
    ///
    /// The order of fields inside the container is fixed
    #[inline]
    pub fn new<T: TypePath + TupleStruct>(fields: &[UnnamedField]) -> Self {
        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            fields: fields.to_vec().into_boxed_slice(),
            custom_attributes: Arc::new(CustomAttributes::default()),
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
