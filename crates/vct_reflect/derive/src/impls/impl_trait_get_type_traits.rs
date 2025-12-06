use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::Ident;
use crate::derive_data::ReflectMeta;

/// vec_reflect::registry::GetTypeTraits
pub(crate) fn impl_trait_get_type_traits<'a>(meta: &ReflectMeta, register_deps_tokens: TokenStream) -> TokenStream {
    debug_assert!(meta.attrs().impl_switchs.impl_get_type_traits);

    let vct_reflect_path = meta.vct_reflect_path();
    let get_type_traits_ = crate::path::get_type_traits_(vct_reflect_path);
    let type_traits_ = crate::path::type_traits_(vct_reflect_path);
    let from_type_ = crate::path::from_type_(vct_reflect_path);
    let type_trait_from_ptr = crate::path::type_trait_from_ptr_(vct_reflect_path);
    let type_trait_from_reflect = crate::path::type_trait_from_reflect_(vct_reflect_path);

    let outer_ = Ident::new("__outer", Span::call_site());
    
    let insert_default = match meta.attrs().avail_traits.default {
        Some(span) => {
            let type_trait_default_ = crate::path::type_trait_default_(vct_reflect_path);
            quote_spanned! { span =>
                #type_traits_::insert::<#type_trait_default_>(&mut #outer_, #from_type_::<Self>::from_type());
            }
        },
        None => crate::utils::empty(),
    };

    let insert_serialize = match meta.attrs().avail_traits.serialize {
        Some(span) => {
            let type_trait_serialize_ = crate::path::type_trait_serialize_(vct_reflect_path);
            quote_spanned! { span =>
                #type_traits_::insert::<#type_trait_serialize_>(&mut #outer_, #from_type_::<Self>::from_type());
            }
        },
        None => crate::utils::empty(),
    };
    
    let insert_deserialize = match meta.attrs().avail_traits.deserialize {
        Some(span) => {
            let type_trait_deserialize_ = crate::path::type_trait_deserialize_(vct_reflect_path);
            quote_spanned! { span =>
                #type_traits_::insert::<#type_trait_deserialize_>(&mut #outer_, #from_type_::<Self>::from_type());
            }
        },
        None => crate::utils::empty(),
    };

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #get_type_traits_ for #real_ident #ty_generics #where_clause {
            fn get_type_traits() -> #type_traits_ {
                let mut #outer_ = #type_traits_::of::<Self>();
                #type_traits_::insert::<#type_trait_from_ptr>(&mut #outer_, #from_type_::<Self>::from_type());
                #type_traits_::insert::<#type_trait_from_reflect>(&mut #outer_, #from_type_::<Self>::from_type());
                #insert_default
                #insert_serialize
                #insert_deserialize
                #outer_
            }

            #register_deps_tokens
        }
    }
}
