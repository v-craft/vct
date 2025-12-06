use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, quote};
use syn::Ident;

use crate::{ReflectMeta, derive_data::{FieldAccessors, ReflectStruct}, impls::{get_common_debug_impl, get_common_hash_impl, get_common_partial_eq_impl, get_struct_clone_impl, impl_struct_from_reflect, impl_trait_get_type_traits, impl_trait_reflect, impl_trait_type_path, impl_trait_typed}};



pub(crate) fn impl_tuple_struct(info: &ReflectStruct) -> TokenStream {
    let meta = info.meta();

    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, info.to_info_tokens(true))
    } else {
        crate::utils::empty()
    };

    // trait: TupleStruct
    let tuple_struct_trait_tokens = if meta.attrs().impl_switchs.impl_tuple_struct {
        impl_trait_tuple_struct(info)
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().impl_switchs.impl_reflect {
        let try_apply_tokens = get_tuple_struct_try_apply_impl(meta);
        let to_dynamic_tokens = get_tuple_struct_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_struct_clone_impl(info);
        let reflect_partial_eq_tokens = get_common_partial_eq_impl(meta);
        let reflect_hash_tokens = get_common_hash_impl(meta);
        let reflect_debug_tokens = get_common_debug_impl(meta);

        impl_trait_reflect(
            meta, 
            quote!(TupleStruct),
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
        impl_trait_get_type_traits(meta, get_registry_dependencies(info))
    } else {
        crate::utils::empty()
    };

    // trait: FromReflect
    let get_from_reflect_tokens = if meta.attrs().impl_switchs.impl_from_reflect {
        impl_struct_from_reflect(info, true)
    } else {
        crate::utils::empty()
    };

    quote! {
        #type_path_trait_tokens

        #typed_trait_tokens

        #tuple_struct_trait_tokens

        #reflect_trait_tokens

        #get_type_traits_tokens

        #get_from_reflect_tokens
    }
}

pub fn impl_trait_tuple_struct(info: &ReflectStruct) -> TokenStream {
    use crate::path::fp::OptionFP;
    let meta = info.meta();
    
    let vct_reflect_path = meta.vct_reflect_path();
    let tuple_struct_ = crate::path::tuple_struct_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let tuple_struct_field_iter_ = crate::path::tuple_struct_field_iter_(vct_reflect_path);
    let dynamic_tuple_struct_ = crate::path::dynamic_tuple_struct_(vct_reflect_path);
    let option_ = OptionFP.to_token_stream();

    let FieldAccessors {
        fields_ref,
        fields_mut,
        field_indices,
        field_count,
    } = FieldAccessors::new(info);

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #tuple_struct_ for #real_ident #ty_generics #where_clause {
            fn field(&self, index: usize) -> #OptionFP<&dyn #reflect_> {
                match index {
                    #(#field_indices => #option_::Some(#fields_ref),)*
                    _ => #OptionFP::None,
                }
            }

            fn field_mut(&mut self, index: usize) -> #OptionFP<&mut dyn #reflect_> {
                match index {
                    #(#field_indices => #option_::Some(#fields_mut),)*
                    _ => #OptionFP::None,
                }
            }
            #[inline]
            fn field_len(&self) -> usize {
                #field_count
            }
            #[inline]
            fn iter_fields(&self) -> #tuple_struct_field_iter_ {
                #tuple_struct_field_iter_::new(self)
            }

            fn to_dynamic_tuple_struct(&self) -> #dynamic_tuple_struct_ {
                let mut dynamic = #dynamic_tuple_struct_::with_capacity(#tuple_struct_::field_len(self));
                dynamic.set_type_info(#reflect_::represented_type_info(self));
                #(dynamic.insert_boxed(#reflect_::to_dynamic(#fields_ref));)*
                dynamic
            }
        }
    }
}

pub fn get_tuple_struct_try_apply_impl(meta: &ReflectMeta) -> TokenStream {
    use crate::path::fp::{ResultFP, OptionFP};

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_ref = crate::path::reflect_ref_(vct_reflect_path);
    let tuple_struct_ = crate::path::tuple_struct_(vct_reflect_path);
    let reflect_kind_ = crate::path::reflect_kind_(vct_reflect_path);
    let apply_error_ = crate::path::apply_error_(vct_reflect_path);

    let input_ = Ident::new("__ident", Span::call_site());

    quote! {
        fn try_apply(&mut self, #input_: &dyn #reflect_) -> #ResultFP<(), #apply_error_> {
            if <dyn #reflect_>::is::<Self>(#input_) {
                if let Ok(cloned) = #reflect_::reflect_clone(#input_)
                    && let Ok(__val) = <dyn #reflect_>::take::<Self>(cloned)
                {
                    *self = __val;
                    return #ResultFP::Ok(())
                }
            }

            if let #reflect_ref::TupleStruct(struct_value) = #reflect_::reflect_ref(#input_) {
                for (i, value) in ::core::iter::Iterator::enumerate(#tuple_struct_::iter_fields(struct_value)) {
                    if let #OptionFP::Some(v) = #tuple_struct_::field_mut(self, i) {
                        #reflect_::try_apply(v, value)?;
                    }
                }

                #ResultFP::Ok(())
            } else {
                #ResultFP::Err(
                    #apply_error_::MismatchedKinds {
                        from_kind: #reflect_::reflect_kind(#input_),
                        to_kind: #reflect_kind_::TupleStruct,
                    }
                )
            }
        }
    }
}

fn get_tuple_struct_to_dynamic_impl(meta: &ReflectMeta) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let tuple_struct_ = crate::path::tuple_struct_(vct_reflect_path);

    quote! {
        #[inline]
        fn to_dynamic(&self) -> #alloc_utils_::Box<dyn #reflect_> {
            #alloc_utils_::Box::new( #tuple_struct_::to_dynamic_tuple_struct(self) )
        }
    }
}

fn get_registry_dependencies(info: &ReflectStruct) -> TokenStream {
    let vct_reflect_path = info.meta().vct_reflect_path();
    let type_registry_ = crate::path::type_registry_(vct_reflect_path);

    let field_types =  info.active_fields().map(|x|&x.data.ty);

    quote! {
        fn register_dependencies(__registry: &mut #type_registry_) {
            #(#type_registry_::register::<#field_types>(__registry);)*
        }
    }
}

