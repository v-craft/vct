use syn::Path;

use crate::derive_data::{
    TypeAttributes, ReflectTypePath,
};

pub(crate) struct ReflectMeta<'a> {
    /// The registered traits for this type.
    attrs: TypeAttributes,
    /// The path to this type.
    type_path: ReflectTypePath<'a>,
    /// A cached instance of the path to the `vct_reflect` crate.
    vct_reflect_path: Path,
}

impl<'a> ReflectMeta<'a> {
    pub fn new(attrs: TypeAttributes, type_path: ReflectTypePath<'a>) -> Self {
        Self {
            attrs,
            type_path,
            vct_reflect_path: crate::path::vct_reflect(),
        }
    }

    pub fn vct_reflect_path(&self) -> &Path {
        &self.vct_reflect_path
    }

    pub fn type_path(&self) -> &ReflectTypePath {
        &self.type_path
    }

    pub fn attrs(&self) -> &TypeAttributes {
        &self.attrs
    }

    

}

