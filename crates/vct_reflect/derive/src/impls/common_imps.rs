use crate::ReflectMeta;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

/// Try `clone` or `reflect_clone`
pub(crate) fn get_common_try_apply_tokens(meta: &ReflectMeta, input: &syn::Ident) -> TokenStream {
    use crate::path::fp::{ResultFP, CloneFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    match meta.attrs().avail_traits.clone {
        Some(span) => quote_spanned! { span =>
            if let Some(__val) = <dyn #reflect_>::downcast_ref::<Self>(#input) {
                *self = #CloneFP::clone(__val);
                return #ResultFP::Ok(());
            }
        },
        None => quote! {
            if <dyn #reflect_>::is::<Self>(#input) {
                if let Ok(__cloned) = #reflect_::reflect_clone(#input)
                    && Ok(__val) = <dyn #reflect_>::take::<Self>(__cloned)
                {
                    *self = __val;
                    return #ResultFP::Ok(());
                }
            }
        },
    }
}

/// Try `clone` or `reflect_clone`
pub(crate) fn get_common_from_reflect_tokens(meta: &ReflectMeta, input: &syn::Ident) -> TokenStream {
    use crate::path::fp::{OptionFP, CloneFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    match meta.attrs().avail_traits.clone {
        Some(span) => quote_spanned! { span =>
            if let Some(__val) = <dyn #reflect_>::downcast_ref::<Self>(#input) {
                return #OptionFP::Some(#CloneFP::clone(__val));
            }
        },
        None => quote! {
            if <dyn #reflect_>::is::<Self>(#input) {
                if let Ok(__cloned) = #reflect_::reflect_clone(#input)
                    && Ok(__val) = <dyn #reflect_>::take::<Self>(__cloned)
                {
                    return #OptionFP::Some(__val);
                }
            }
        },
    }
}

pub(crate) fn get_common_partial_eq_impl(meta: &ReflectMeta) -> TokenStream  {
    use crate::path::fp::{OptionFP, PartialEqFP};
    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.partial_eq {
        quote_spanned! { span =>
            #[inline]
            fn reflect_partial_eq(&self, other: &dyn #reflect_) -> #OptionFP<bool> {
                if let #OptionFP::Some(value) = other.downcast_ref::<Self>() {
                    return #OptionFP::Some( #PartialEqFP::eq(self, value) );
                }
                #OptionFP::None
            }
        }
    } else {
        crate::utils::empty()
    }
}

pub(crate) fn get_common_hash_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::{OptionFP, HashFP, HasherFP};
    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_hasher = crate::path::reflect_hasher_(vct_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.hash {
        quote_spanned! { span =>
            #[inline]
            fn reflect_hash(&self) -> #OptionFP<u64> {
                let mut hasher = #reflect_hasher();
                <Self as #HashFP>::hash(self, &mut hasher);
                #OptionFP::Some(#HasherFP::finish(&hasher))
            }
        }
    } else {
        crate::utils::empty()
    }
}

pub(crate) fn get_common_debug_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::DebugFP;

    if let Some(span) = meta.attrs().avail_traits.debug {
        quote_spanned! { span =>
            #[inline]
            fn reflect_debug(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                <Self as #DebugFP>::fmt(self, f)
            }
        }
    } else {
        crate::utils::empty()
    }
}


