use syn::Field;
use crate::derive_data::{FieldAttributes, ReflectMeta};


pub(crate) struct ReflectStruct<'a> {
    meta: ReflectMeta<'a>,
    fields: Vec<StructField<'a>>,
}

#[derive(Clone)]
pub(crate) struct StructField<'a> {
    pub data: &'a Field,
    pub attrs: FieldAttributes,
    pub declaration_index: usize,
    /// This index accounts for the removal of [ignored] fields.
    pub reflection_index: Option<usize>,
}
