//! This independent module is used to provide the required path.
//! So as to minimize changes when the `vct_reflect` structure is modified.
//! 
//! The only special feature is the path of vct_reflect itself,
//! See [`vct_reflect`] function doc.


use proc_macro2::TokenStream;
use quote::quote;

/// Get the correct access path to the `vct_reflect` crate.
/// 
/// Not all modules can access the reflection crate itself through `vct_reflect`,
/// we have to scan the builder's `cargo.toml`.
/// 
/// 1. For the `vct_reflect` crate itself, `crate` is returned here`.
/// 2. For crates that depend on `vct_reflect`, `vct_reflect` is returned here`.
/// 3. For crates that depend on `vct`, `vct::reflect` is returned here`.
/// 4. For other situations, `vct::reflect` is returned here`, but this may be incorrect.
/// 
/// The cost of this function is relatively high (accessing files, obtaining read-write lock permissions, querying content...),
/// so the crate path is mainly obtained through parameter passing rather than reacquiring.
pub(crate) fn vct_reflect() -> syn::Path {
    vct_macro_utils::Manifest::shared(|manifest|manifest.get_path("vct_reflect"))
}

pub(crate) mod fp;
mod cell;
mod info;
mod ops;
mod registry;

pub(crate) use cell::*;
pub(crate) use info::*;
pub(crate) use ops::*;
pub(crate) use registry::*;

// mod access;
// `vct_reflect::access` does not require additional content.

#[inline(always)]
pub(crate) fn macro_exports_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::__macro_exports
    }
}

#[inline(always)]
pub(crate) fn alloc_utils_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::__macro_exports::alloc_utils
    }
}

#[inline(always)]
pub(crate) fn reflect_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::Reflect
    }
}

#[inline(always)]
pub(crate) fn from_reflect_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::FromReflect
    }
}

#[inline(always)]
pub(crate) fn reflect_hasher_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::reflect_hasher
    }
}

// #[inline(always)]
// pub(crate) fn reflectable_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::Reflectable
//     }
// }


