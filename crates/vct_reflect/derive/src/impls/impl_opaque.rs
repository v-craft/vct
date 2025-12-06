use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::Ident;

use crate::{derive_data::ReflectMeta, impls::{get_common_debug_impl, get_common_from_reflect_tokens, get_common_hash_impl, get_common_partial_eq_impl, impl_trait_get_type_traits, impl_trait_reflect, impl_trait_type_path, impl_trait_typed}};

pub(crate) fn impl_opaque(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, meta.to_info_tokens())
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().impl_switchs.impl_reflect {
        let try_apply_tokens = get_opaque_try_apply_impl(meta);
        let to_dynamic_tokens = get_opaque_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_opaque_clone_impl(meta);
        let reflect_partial_eq_tokens = get_common_partial_eq_impl(meta);
        let reflect_hash_tokens = get_common_hash_impl(meta);
        let reflect_debug_tokens = get_common_debug_impl(meta);

        impl_trait_reflect(
            meta, 
            quote!(Opaque),
            try_apply_tokens,
            to_dynamic_tokens,
            reflect_clone_tokens,
            reflect_partial_eq_tokens,
            reflect_hash_tokens,
            reflect_debug_tokens,
        )
    } else {
        crate::utils::empty()
    };

    // trait: GetTypeTraits
    let get_type_traits_tokens = if meta.attrs().impl_switchs.impl_get_type_traits {
        impl_trait_get_type_traits(meta, crate::utils::empty())
    } else {
        crate::utils::empty()
    };

    // trait: FromReflect
    let from_reflect_tokens = if meta.attrs().impl_switchs.impl_from_reflect {
        impl_opaque_from_reflect(meta)
    } else {
        crate::utils::empty()
    };

    quote! {
        #type_path_trait_tokens

        #typed_trait_tokens

        #reflect_trait_tokens

        #get_type_traits_tokens

        #from_reflect_tokens
    }
}

fn get_opaque_try_apply_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{ResultFP, OptionFP, CloneFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let apply_error_ = crate::path::apply_error_(vct_reflect_path);
    let type_path_ = crate::path::type_path_(vct_reflect_path);
    let dynamic_type_path_ = crate::path::dynamic_type_path_(vct_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            fn try_apply(&mut self, __input: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
                if let #OptionFP::Some(__input) = <dyn #reflect_>::downcast_ref::<Self>(__input) {
                    *self = #CloneFP::clone(__input);
                    return #ResultFP::Ok(());
                }

                #ResultFP::Err(
                    #apply_error_::MismatchedTypes {
                        from_type: #alloc_utils_::Cow::Owned(#alloc_utils_::ToOwned::to_owned(#dynamic_type_path_::reflect_type_path(__input))),
                        to_type: #alloc_utils_::Cow::Borrowed(<Self as #type_path_>::type_path()),
                    }
                )
            }
        }
    } else {
        unreachable!("#[reflect(clone)] must be specified when auto impl `Reflect` for Opaque Type.")
    }
}

fn get_opaque_to_dynamic_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::CloneFP;

    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            #[inline]
            fn to_dynamic(&self) -> #alloc_utils_::Box<dyn #reflect_> {
                #alloc_utils_::Box::new(<Self as #CloneFP>::clone(self))
            }
        }
    } else {
        unreachable!("#[reflect(clone)] must be specified when auto impl `Reflect` for Opaque Type.")
    }
}

fn get_opaque_clone_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{ResultFP, CloneFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vct_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#alloc_utils_::Box::new(<Self as #CloneFP>::clone(self)) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }
    } else {
        unreachable!("#[reflect(clone)] must be specified when auto impl `Reflect` for Opaque Type.")
    }
}

fn impl_opaque_from_reflect(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::OptionFP;

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let from_reflect_ = crate::path::from_reflect_(vct_reflect_path);

    let input_ = Ident::new("__input", Span::call_site());

    let clone_tokens = get_common_from_reflect_tokens(meta, &input_);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #from_reflect_ for #real_ident #ty_generics #where_clause  {
            fn from_reflect(#input_: &dyn #reflect_) -> #OptionFP<Self> {
                #clone_tokens

                #OptionFP::None
            }
        }
    }
}
