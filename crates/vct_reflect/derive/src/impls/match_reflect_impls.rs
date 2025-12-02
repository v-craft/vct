use proc_macro::TokenStream;
use syn::{DeriveInput, spanned::Spanned};
use crate::{ImplSourceKind, derive_data::{ReflectDerive, ReflectMeta, ReflectTypePath, TypeAttributes}};

pub(crate) fn match_reflect_impls(ast: DeriveInput, source: ImplSourceKind) -> TokenStream {
    let reflect_derive = match ReflectDerive::from_input(&ast, source) {
        Ok(val) => val,
        Err(err) => return err.into_compile_error().into(),
    };


    



    todo!()
}

