use proc_macro2::TokenStream;
use quote::quote;
use crate::{
    path::fp::OptionFP,
    derive_data::ReflectMeta, utils::{StringExpr , wrap_in_option}
};

fn static_path_cell(vct_reflect_path: &syn::Path, generator: TokenStream) -> TokenStream {
    let cell_path = crate::path::generic_type_path_cell_(vct_reflect_path);

    quote! {
        static CELL: #cell_path = #cell_path::new()
        CELL.get_or_insert::<Self, _>(|| {
            #generator
        })
    }
}

pub(crate) fn impl_trait_type_path(meta: &ReflectMeta) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();
    let trait_type_path_ = crate::path::type_path_(vct_reflect_path);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();

    let inline_flag = if parser.impl_with_generic() {
        crate::utils::empty()
    } else {
        quote! { #[inline] }
    };

    let (type_path, type_name) = if parser.impl_with_generic() {
        (
            static_path_cell(vct_reflect_path, parser.type_path_into_owned(vct_reflect_path)),
            static_path_cell(vct_reflect_path, parser.type_name_into_owned(vct_reflect_path)),
        )
    } else {
        (
            parser.type_path(vct_reflect_path).into_borrowed(),
            parser.type_name(vct_reflect_path).into_borrowed(),
        )
    };

    // Only Primitive type can return None
    let type_ident = parser.type_ident().into_borrowed();
    let module_path = wrap_in_option(parser.module_path().map(StringExpr::into_borrowed));
    let crate_name = wrap_in_option(parser.crate_name().map(StringExpr::into_borrowed));

    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #trait_type_path_ for #real_ident #ty_generics #where_clause {
            #inline_flag
            fn type_path() -> &'static str {
                #type_path
            }

            #inline_flag
            fn type_name() -> &'static str {
                #type_name
            }

            #[inline]
            fn type_ident() -> &'static str {
                #type_ident
            }

            #[inline]
            fn crate_name() -> #OptionFP<&'static str> {
                #crate_name
            }

            #[inline]
            fn module_path() -> #OptionFP<&'static str> {
                #module_path
            }
        }
    }
}
