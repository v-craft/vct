use proc_macro2::TokenStream;
use quote::{quote_spanned, quote};

use crate::derive_data::ReflectStruct;

pub(crate) fn get_struct_clone_impl(info: &ReflectStruct) -> TokenStream {
    use crate::path::fp::{ResultFP, CloneFP, OptionFP, DefaultFP};

    let meta = info.meta();
    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let macro_exports_ = crate::path::macro_exports_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vct_reflect_path);
    let type_path_ = crate::path::type_path_(vct_reflect_path);


    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#alloc_utils_::Box::new(<Self as #CloneFP>::clone(self)) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }
    } else if let Some(span) = meta.attrs().avail_traits.default {
        let mut tokens = TokenStream::new();

        for field in info.active_fields() {
            let field_ty = &field.data.ty;
            let member = field.to_member();

            tokens.extend(quote! {
                __new_value.#member = #macro_exports_::reflect_clone_field::<#field_ty>(&self.#member)?;
            });
        }

        quote_spanned! { span =>
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                let mut __new_value = <Self as #DefaultFP>::default();

                #tokens

                #ResultFP::Ok(#alloc_utils_::Box::new(__new_value) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }

    } else {
        for field in info.fields().iter() {
            if let Some(span) = field.attrs.ignore {
                let field_id = field.field_id(vct_reflect_path);
                return quote_spanned! { span =>
                    #[inline]
                    fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                        #ResultFP::Err(#reflect_clone_error_::FieldNotCloneable {
                            type_path:  #alloc_utils_::Cow::Borrowed(<Self as #type_path_>::type_path())
                            field: #field_id,
                            variant: #OptionFP::None,
                        })
                    }
                };
            }
        }

        let mut tokens = TokenStream::new();

        for field in info.fields().iter() {
            let field_ty = &field.data.ty;
            let member = field.to_member();

            tokens.extend(quote! {
                #member: #macro_exports_::reflect_clone_field::<#field_ty>(&self.#member)?,
            });
        }

        quote! {
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#alloc_utils_::Box::new(
                    Self {
                        #tokens
                    }
                ) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }
    }
}

