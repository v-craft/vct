use proc_macro2::TokenStream;
use quote::quote;
use crate::derive_data::ReflectMeta;



pub(crate) fn impl_trait_get_type_traits<'a>(meta: &ReflectMeta, register_deps_tokens: TokenStream) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();

    let get_type_traits_ = crate::path::get_type_traits_(vct_reflect_path);
    let type_traits_ = crate::path::type_traits_(vct_reflect_path);
    let from_type_ = crate::path::from_type_(vct_reflect_path);
    let type_trait_from_ptr = crate::path::type_trait_from_ptr_(vct_reflect_path);
    let type_trait_from_reflect = crate::path::type_trait_from_reflect_(vct_reflect_path);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #get_type_traits_ for #real_ident #ty_generics #where_clause {
            fn get_type_traits() -> #type_traits_ {
                let mut type_traits = #type_traits_::of::<Self>();
                type_traits.insert::<#type_trait_from_ptr>(#from_type_::<Self>::from_type());
                type_traits.insert::<#type_trait_from_reflect>(#from_type_::<Self>::from_type());
                type_traits
            }

            #register_deps_tokens
        }
    }
}
