use quote::{quote, quote_spanned};

use crate::{derive_data::{MethodFlag, ReflectMeta}, impls::{impl_trait_get_type_traits, impl_trait_reflect, impl_trait_type_path, impl_trait_typed}};

pub(crate) fn impl_opaque(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().trait_flags.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().trait_flags.impl_typed {
        impl_trait_typed(meta, meta.to_info_tokens())
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().trait_flags.impl_reflect {
        let try_apply_tokens = get_opaque_try_apply_impl(meta);
        let to_dynamic_tokens = get_opaque_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_opaque_clone_impl(meta);
        let reflect_partial_eq_tokens = get_opaque_partial_eq_impl(meta);
        let reflect_hash_tokens = get_opaque_hash_impl(meta);
        let reflect_debug_tokens = get_opaque_debug_impl(meta);

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
    let get_type_traits_tokens = if meta.attrs().trait_flags.impl_get_type_traits {
        impl_trait_get_type_traits(meta, crate::utils::empty())
    } else {
        crate::utils::empty()
    };

    // trait: FromReflect
    let from_reflect_tokens = if meta.attrs().trait_flags.impl_from_reflect {
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

    quote! {
        fn try_apply(&mut self, value: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
            if let #OptionFP::Some(value) = value.downcast_ref::<Self>() {
                *self = #CloneFP::clone(value);
                return #ResultFP::Ok(());
            }

            #ResultFP::Err(
                #apply_error_::MismatchedTypes {
                    from_type: #alloc_utils_::ToString::to_string(#dynamic_type_path_::reflect_type_path(value)),
                    to_type: #alloc_utils_::ToString::to_string(<Self as #type_path_>::type_path()),
                }
            )
        }
    }
}

fn get_opaque_to_dynamic_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::CloneFP;

    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    quote! {
        #[inline]
        fn to_dynamic(&self) -> #alloc_utils_::Box<dyn #reflect_> {
            #alloc_utils_::Box::new(<Self as #CloneFP>::clone(self))
        }
    }
}

fn get_opaque_clone_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{ResultFP, CloneFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vct_reflect_path);

    match meta.attrs().method_flags.reflect_clone.clone() {
        MethodFlag::Default => {
            quote! {
                #[inline]
                fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                    #ResultFP::Ok(#alloc_utils_::Box::new(<Self as #CloneFP>::clone(self)).into_reflect())
                }
            }
        },
        MethodFlag::Custom(path, span) => {
            quote_spanned! {span =>
                #[inline]
                fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                    #ResultFP::Ok(#alloc_utils_::Box::new(#path(self)).into_reflect())
                }
            }
        },
        _ => unreachable!("`reflect_clone` flag can only be Default or Custom."),
    }
}

fn get_opaque_partial_eq_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream  {
    use crate::path::fp::{OptionFP, PartialEqFP};
    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    match meta.attrs().method_flags.reflect_partial_eq.clone() {
        MethodFlag::Default => crate::utils::empty(),
        MethodFlag::Internal => crate::utils::empty(),
        MethodFlag::Trait(span) => {
            quote_spanned! {span =>
                #[inline]
                fn reflect_partial_eq(&self, other: &dyn #reflect_) -> #OptionFP<bool> {
                    if let #OptionFP::Some(value) = other.downcast_ref::<Self>() {
                        return #OptionFP::Some( #PartialEqFP::eq(self, value) );
                    }
                    #OptionFP::None
                }
            }
        },
        MethodFlag::Custom(path, span) => {
            quote_spanned! {span =>
                #[inline]
                fn reflect_partial_eq(&self, other: &dyn #reflect_) -> #OptionFP<bool> {
                    if let #OptionFP::Some(value) = other.downcast_ref::<Self>() {
                        return #OptionFP::Some( #path(self, value) );
                    }
                    #OptionFP::None
                }
            }
        },
    }
}

fn get_opaque_hash_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{OptionFP, HashFP, HasherFP};
    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_hasher = crate::path::reflect_hasher_(vct_reflect_path);

    match meta.attrs().method_flags.reflect_hash.clone() {
        MethodFlag::Default => crate::utils::empty(),
        MethodFlag::Internal => crate::utils::empty(),
        MethodFlag::Trait(span) => {
            quote_spanned! {span =>
                fn reflect_hash(&self) -> #OptionFP<u64> {
                    let mut hasher = #reflect_hasher();
                    <Self as #HashFP>::hash(self, &mut hasher);
                    #OptionFP::Some(#HasherFP::finish(&hasher))
                }
            }
        },
        MethodFlag::Custom(path, span) => {
            quote_spanned! {span =>
                fn reflect_hash(&self) -> #OptionFP<u64> {
                    #OptionFP::Some( #path(self) )
                }
            }
        },
    }
}

fn get_opaque_debug_impl(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::DebugFP;

    match meta.attrs().method_flags.reflect_debug.clone() {
        MethodFlag::Default => crate::utils::empty(),
        MethodFlag::Internal => crate::utils::empty(),
        MethodFlag::Trait(span) => {
            quote_spanned! {span =>
                #[inline]
                fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #DebugFP::fmt(self, f)
                }
            }
        },
        MethodFlag::Custom(path, span) => {
            quote_spanned! {span =>
                #[inline]
                fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #path(self, f)
                }
            }
        },
    }
}

fn impl_opaque_from_reflect(meta: &ReflectMeta) -> proc_macro2::TokenStream {
    use crate::path::fp::{OptionFP, CloneFP};
    let vct_reflect_path = meta.vct_reflect_path();
    let from_reflect_ = crate::path::from_reflect_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #from_reflect_ for #real_ident #ty_generics #where_clause  {
            fn from_reflect(value: &dyn #reflect_) -> #OptionFP<Self> {
                #OptionFP::Some(#CloneFP::clone(
                    value.downcast_ref::<Self>()?
                ))
            }
        }
    }
}
