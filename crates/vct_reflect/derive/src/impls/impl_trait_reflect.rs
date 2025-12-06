use proc_macro2::TokenStream;
use quote::quote;

use crate::derive_data::ReflectMeta;

pub(crate) fn impl_trait_reflect(
    meta: &ReflectMeta,
    reflect_kind_token: TokenStream,
    try_apply_tokens: TokenStream,
    to_dynamic_tokens: TokenStream,
    reflect_clone_tokens: TokenStream,
    reflect_partial_eq_tokens: TokenStream,
    reflect_hash_tokens: TokenStream,
    reflect_debug_tokens: TokenStream,
) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();

    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_kind_ = crate::path::reflect_kind_(vct_reflect_path);
    let reflect_ref_ = crate::path::reflect_ref_(vct_reflect_path);
    let reflect_mut_ = crate::path::reflect_mut_(vct_reflect_path);
    let reflect_owned_ = crate::path::reflect_owned_(vct_reflect_path);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #reflect_ for #real_ident #ty_generics #where_clause {
            #[inline]
            fn as_reflect(&self) -> &dyn #reflect_ {
                self
            }

            #[inline]
            fn as_reflect_mut(&mut self) -> &mut dyn #reflect_ {
                self
            }

            #[inline]
            fn into_reflect(self: #alloc_utils_::Box<Self>) -> #alloc_utils_::Box<dyn #reflect_> {
                self
            }

            #[inline]
            fn set(&mut self, value: #alloc_utils_::Box<dyn #reflect_>) -> ::core::result::Result<(), #alloc_utils_::Box<dyn #reflect_>> {
                *self = value.take::<Self>()?;
                Ok(())
            }

            #[inline]
            fn reflect_kind(&self) -> #reflect_kind_ {
                #reflect_kind_::#reflect_kind_token
            }

            #[inline]
            fn reflect_ref(&self) -> #reflect_ref_<'_> {
                #reflect_ref_::#reflect_kind_token(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> #reflect_mut_<'_> {
                #reflect_mut_::#reflect_kind_token(self)
            }

            #[inline]
            fn reflect_owned(self: #alloc_utils_::Box<Self>) -> #reflect_owned_ {
                #reflect_owned_::#reflect_kind_token(self)
            }

            #to_dynamic_tokens

            #try_apply_tokens

            #reflect_clone_tokens

            #reflect_partial_eq_tokens

            #reflect_hash_tokens

            #reflect_debug_tokens
        }
    }
}

