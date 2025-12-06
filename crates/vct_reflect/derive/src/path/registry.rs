use proc_macro2::TokenStream;
use quote::quote;

// #[inline]
// pub(crate) fn type_trait_(vct_reflect_path: &syn::Path) -> TokenStream { 
//     quote! {
//         #vct_reflect_path::registry::TypeTrait
//     }
// }
    
#[inline]
pub(crate) fn type_traits_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::TypeTraits
    }
}

#[inline]
pub(crate) fn get_type_traits_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::GetTypeTraits
    }
}

#[inline]
pub(crate) fn from_type_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::FromType
    }
}

#[inline]
pub(crate) fn type_registry_(vct_reflect_path: &syn::Path) -> TokenStream {    
    quote! {
        #vct_reflect_path::registry::TypeRegistry
    }
}
    
// #[inline]
// pub(crate) fn type_registry_arc_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::registry::TypeRegistryArc
//     }
// }

#[inline]
pub(crate) fn type_trait_default_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::TypeTraitDefault
    }
}

#[inline]
pub(crate) fn type_trait_from_ptr_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::TypeTraitFromPtr
    }
}

#[inline]
pub(crate) fn type_trait_from_reflect_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::TypeTraitFromReflect
    }
}

#[inline]
pub(crate) fn type_trait_serialize_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::TypeTraitSerialize
    }
}

#[inline]
pub(crate) fn type_trait_deserialize_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::registry::TypeTraitDeserialize
    }
}
