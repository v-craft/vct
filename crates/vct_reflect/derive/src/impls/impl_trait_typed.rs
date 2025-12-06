use proc_macro2::TokenStream;
use quote::quote;
use crate::derive_data::ReflectMeta;

pub(crate) fn impl_trait_typed(meta: &ReflectMeta, type_info_tokens: TokenStream) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();
    let trait_typed_ = crate::path::typed_(vct_reflect_path);
    let type_info_ = crate::path::type_info_(vct_reflect_path);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    
    let inner_cell_tokens = if parser.impl_with_generic() {
        let info_cell = crate::path::generic_type_info_cell_(vct_reflect_path);
        quote! {
            static CELL: #info_cell = #info_cell::new();
            CELL.get_or_insert::<Self, _>(|| {
                #type_info_tokens
            })
        }
    } else {
        let info_cell = crate::path::non_generic_type_info_cell_(vct_reflect_path);
        quote! {
            static CELL: #info_cell = #info_cell::new();
            CELL.get_or_init(|| {
                #type_info_tokens
            })
        }
    };

    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #trait_typed_ for #real_ident #ty_generics #where_clause {
            fn type_info() -> &'static #type_info_ {
                #inner_cell_tokens
            }
        }
    }
}
