use proc_macro2::{TokenStream, Span};
use quote::{quote, quote_spanned};
use syn::Ident;
use crate::{ReflectMeta, derive_data::{EnumVariantFields, ReflectEnum, StructField}, impls::{get_common_debug_impl, get_common_from_reflect_tokens, get_common_hash_impl, get_common_partial_eq_impl, get_common_try_apply_tokens, impl_trait_get_type_traits, impl_trait_reflect, impl_trait_type_path, impl_trait_typed}};



pub(crate) fn impl_enum(info: &ReflectEnum) -> TokenStream {
    let meta = info.meta();

    // trait: TypePath
    let type_path_trait_tokens = if meta.attrs().impl_switchs.impl_type_path {
        impl_trait_type_path(meta)
    } else {
        crate::utils::empty()
    };
    
    // trait: Typed
    let typed_trait_tokens = if meta.attrs().impl_switchs.impl_typed {
        impl_trait_typed(meta, info.to_info_tokens())
    } else {
        crate::utils::empty()
    };

    // trait: Enum
    let enum_trait_tokens = if meta.attrs().impl_switchs.impl_enum {
        impl_trait_enum(info)
    } else {
        crate::utils::empty()
    };

    // trait: Reflect
    let reflect_trait_tokens = if meta.attrs().impl_switchs.impl_reflect {
        let try_apply_tokens = get_enum_try_apply_impl(info);
        let to_dynamic_tokens = get_enum_to_dynamic_impl(meta);
        let reflect_clone_tokens = get_enum_clone_impl(info);
        let reflect_partial_eq_tokens = get_common_partial_eq_impl(meta);
        let reflect_hash_tokens = get_common_hash_impl(meta);
        let reflect_debug_tokens = get_common_debug_impl(meta);

        impl_trait_reflect(
            meta, 
            quote!(Enum),
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
    let from_reflect_trait_tokens = if meta.attrs().impl_switchs.impl_from_reflect {
        impl_enum_from_reflect(info)
    } else {
        crate::utils::empty()
    };

    quote! {
        #type_path_trait_tokens

        #typed_trait_tokens

        #enum_trait_tokens

        #reflect_trait_tokens

        #get_type_traits_tokens

        #from_reflect_trait_tokens
    }
}

fn impl_trait_enum(info: &ReflectEnum) -> TokenStream {
    use crate::path::fp::OptionFP;
    let meta = info.meta();
    
    let vct_reflect_path = meta.vct_reflect_path();
    let enum_ = crate::path::enum_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let variant_field_iter_ = crate::path::variant_field_iter_(vct_reflect_path);
    let variant_kind_ = crate::path::variant_kind_(vct_reflect_path);

    let ref_name = Ident::new("__name_param", Span::call_site());
    let ref_index = Ident::new("__index_param", Span::call_site());

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    let mut enum_field = Vec::new();
    let mut enum_field_mut = Vec::new();
    let mut enum_field_at = Vec::new();
    let mut enum_field_at_mut = Vec::new();
    let mut enum_index_of = Vec::new();
    let mut enum_name_at = Vec::new();
    let mut enum_field_len = Vec::new();
    let mut enum_variant_name = Vec::new();
    let mut enum_variant_index = Vec::new();
    let mut enum_variant_kind = Vec::new();


    for (variant_index, variant) in info.variants().iter().enumerate() {
        let ident = &variant.data.ident;
        let name = ident.to_string();
        let variant_path_ = info.variant_path(ident);

        let variant_type_ident = match variant.data.fields {
            syn::Fields::Unit => Ident::new("Unit", Span::call_site()),
            syn::Fields::Unnamed(..) => Ident::new("Tuple", Span::call_site()),
            syn::Fields::Named(..) => Ident::new("Struct", Span::call_site()),
        };

        enum_variant_name.push(quote! {
            #variant_path_{..} => #name
        });
        enum_variant_index.push(quote! {
            #variant_path_{..} => #variant_index
        });
        enum_variant_kind.push(quote! {
            #variant_path_{..} => #variant_kind_::#variant_type_ident
        });

        fn process_fields(
            fields: &[StructField],
            mut f: impl FnMut(&StructField) + Sized,
        ) -> usize {
            let mut field_len = 0;
            for field in fields.iter() {
                if field.attrs.ignore.is_some() {
                    continue;
                };
                f(field);
                field_len += 1;
            }
            field_len
        }

        match &variant.fields {
            EnumVariantFields::Unit => {
                enum_field_len.push(quote! {
                    #variant_path_{..} => 0usize
                });
            }
            EnumVariantFields::Unnamed(fields) => {
                let field_len = process_fields(fields, |field: &StructField| {
                    let reflection_index = field.reflection_index.unwrap();

                    let declare_field = syn::Index::from(field.declaration_index);

                    enum_field_at.push(quote! {
                        #variant_path_ { #declare_field : __value, .. } if #ref_index == #reflection_index => #OptionFP::Some(__value)
                    });
                    enum_field_at_mut.push(quote! {
                        #variant_path_ { #declare_field : __value, .. } if #ref_index == #reflection_index => #OptionFP::Some(__value)
                    });
                });

                enum_field_len.push(quote! {
                    #variant_path_{..} => #field_len
                });
            }
            EnumVariantFields::Named(fields) => {
                let field_len = process_fields(fields, |field: &StructField| {
                    let field_ident = field.data.ident.as_ref().unwrap();
                    let field_name = field_ident.to_string();
                    let reflection_index = field.reflection_index.unwrap();

                    enum_field.push(quote! {
                        #variant_path_{ #field_ident: __value, .. } if #ref_name == #field_name => #OptionFP::Some(__value)
                    });
                    enum_field_mut.push(quote! {
                        #variant_path_{ #field_ident: __value, .. } if #ref_name == #field_name => #OptionFP::Some(__value)
                    });
                    enum_field_at.push(quote! {
                        #variant_path_{ #field_ident: __value, .. } if #ref_index == #reflection_index => #OptionFP::Some(__value)
                    });
                    enum_field_at_mut.push(quote! {
                        #variant_path_{ #field_ident: __value, .. } if #ref_index == #reflection_index => #OptionFP::Some(__value)
                    });
                    enum_index_of.push(quote! {
                        #variant_path_{ .. } if #ref_name == #field_name => #OptionFP::Some(#reflection_index)
                    });
                    enum_name_at.push(quote! {
                        #variant_path_{ .. } if #ref_index == #reflection_index => #OptionFP::Some(#field_name)
                    });
                });

                enum_field_len.push(quote! {
                    #variant_path_{..} => #field_len
                });
            }
        };
    }

    quote! {
        impl #impl_generics #enum_ for #real_ident #ty_generics #where_clause {
            fn field(&self, #ref_name: &str) -> #OptionFP<&dyn #reflect_> {
                    match self {
                    #(#enum_field,)*
                    _ => #OptionFP::None,
                }
            }

            fn field_at(&self, #ref_index: usize) -> #OptionFP<&dyn #reflect_> {
                match self {
                    #(#enum_field_at,)*
                    _ => #OptionFP::None,
                }
            }

            fn field_mut(&mut self, #ref_name: &str) -> #OptionFP<&mut dyn #reflect_> {
                    match self {
                    #(#enum_field_mut,)*
                    _ => #OptionFP::None,
                }
            }

            fn field_at_mut(&mut self, #ref_index: usize) -> #OptionFP<&mut dyn #reflect_> {
                match self {
                    #(#enum_field_at_mut,)*
                    _ => #OptionFP::None,
                }
            }

            fn index_of(&self, #ref_name: &str) -> #OptionFP<usize> {
                    match self {
                    #(#enum_index_of,)*
                    _ => #OptionFP::None,
                }
            }

            fn name_at(&self, #ref_index: usize) -> #OptionFP<&str> {
                    match self {
                    #(#enum_name_at,)*
                    _ => #OptionFP::None,
                }
            }

            #[inline]
            fn iter_fields(&self) -> #variant_field_iter_ {
                #variant_field_iter_::new(self)
            }

            #[inline]
            fn field_len(&self) -> usize {
                    match self {
                    #(#enum_field_len,)*
                    _ => 0,
                }
            }

            #[inline]
            fn variant_name(&self) -> &str {
                    match self {
                    #(#enum_variant_name,)*
                    _ => unreachable!(),
                }
            }

            #[inline]
            fn variant_index(&self) -> usize {
                    match self {
                    #(#enum_variant_index,)*
                    _ => unreachable!(),
                }
            }

            #[inline]
            fn variant_kind(&self) -> #variant_kind_ {
                    match self {
                    #(#enum_variant_kind,)*
                    _ => unreachable!(),
                }
            }
        }
    }
}

fn get_enum_try_apply_impl(info: &ReflectEnum) -> TokenStream {
    use crate::path::fp::{ResultFP, OptionFP};

    let meta = info.meta();
    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let enum_ = crate::path::enum_(vct_reflect_path);
    let reflect_kind_ = crate::path::reflect_kind_(vct_reflect_path);
    let reflet_ref = crate::path::reflect_ref_(vct_reflect_path);
    let variant_kind_ = crate::path::variant_kind_(vct_reflect_path);
    let apply_error_ = crate::path::apply_error_(vct_reflect_path);
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let type_path_ = crate::path::type_path_(vct_reflect_path);
    let from_reflect_ = crate::path::from_reflect_(vct_reflect_path);

    let input_ = Ident::new("__input", Span::call_site());

    let clone_tokens = get_common_try_apply_tokens(meta, &input_);

    let mut match_tokens = TokenStream::new();

    for variant in info.variants.iter() {
        let ident = &variant.data.ident;
        let variant_path_ = info.variant_path(ident);
        let variant_name_ = ident.to_string();

        match variant.data.fields {
            syn::Fields::Unit => {
                match_tokens.extend(quote! {
                    #variant_name_ => {
                        *self = #variant_path_;
                        return #ResultFP::Ok(());
                    },
                });
            },
            syn::Fields::Named(..) | syn::Fields::Unnamed(..) => {
                if let Some(field) = variant.fields().iter().find(|f|f.attrs.ignore.is_some()) {
                    let field_name = field.field_name();
                    
                    match_tokens.extend(quote! { span =>
                        #variant_name_ => {
                           return #ResultFP::Err(
                                #apply_error_::MissingEnumField {
                                    variant_name:  #alloc_utils_::Cow::Borrowed(#variant_name_),
                                    field_name: #alloc_utils_::Cow::Borrowed(#field_name),
                                }
                            );
                        },
                    });
                    continue;
                }
                let mut clone_tokens = TokenStream::new();

                for field in variant.fields().iter() {
                    let field_ty = &field.data.ty;
                    let member = field.to_member();
                    let field_name = field.field_name();

                    let accessor = match &field.data.ident {
                        Some(id) => {
                            let name = id.to_string();
                            quote! { #enum_::field(#input_, #name) }
                        },
                        None => {
                            let idx = field.declaration_index;
                            quote! { #enum_::field_at(#input_, #idx) }
                        },
                    };

                    clone_tokens.extend(quote! {
                        #member: {
                            let __other = match #accessor {
                                #OptionFP::Some(__val) => __val,
                                #OptionFP::None => return #ResultFP::Err(
                                    #apply_error_::MissingEnumField {
                                        variant_name:  #alloc_utils_::Cow::Borrowed(#variant_name_),
                                        field_name: #alloc_utils_::Cow::Borrowed(#field_name),
                                    }
                                ),
                            };
                            match <#field_ty as #from_reflect_>::from_reflect(__other) {
                                #OptionFP::Some(__val) => __val,
                                #OptionFP::None => return #ResultFP::Err(
                                    #apply_error_::MissingEnumField {
                                        variant_name:  #alloc_utils_::Cow::Borrowed(#variant_name_),
                                        field_name: #alloc_utils_::Cow::Borrowed(#field_name),
                                    }
                                ),
                            }
                        },
                    });
                }
                match_tokens.extend(quote! {
                    #variant_name_ => {
                        *self = variant_path_{ #clone_tokens };
                    },
                });
                
            },
        }
    }

    quote! {
        fn try_apply(&mut self, #input_: &dyn #reflect_) -> #ResultFP<(), #apply_error_>  {
            #clone_tokens

            if let #reflet_ref::Enum(#input_) = #reflect_::reflect_ref(#input_) {
                if #enum_::variant_name(self) == #enum_::variant_name(#input_) {
                    // Same variant -> just update fields
                    match #enum_::variant_kind(#input_) {
                        #variant_kind_::Struct => {
                            for field in #enum_::iter_fields(#input_) {
                                let name = field.name().unwrap();
                                if let #OptionFP::Some(v) = #enum_::field_mut(self, name) {
                                    #reflect_::try_apply(v, field.value())?;
                                }
                            }
                        },
                        #variant_kind_::Tuple => {
                            for (index, field) in ::core::iter::Iterator::enumerate(#enum_::iter_fields(#input_)) {
                                if let #OptionFP::Some(v) = #enum_::field_at_mut(self, index) {
                                    #reflect_::try_apply(v, field.value())?;
                                }
                            }
                        },
                        _ => {},
                    }
                } else {
                    match #enum_::variant_name(#input_) {
                        #match_tokens
                        __name => {
                            return #ResultFP::Err(
                                #apply_error_::UnknownVariant {
                                    enum_name: #alloc_utils_::Cow::Borrowed(<Self as #type_path_>::type_path()),
                                    variant_name: #alloc_utils_::Cow::Owned(#alloc_utils_::ToOwned::to_owned(__name)),
                                }
                            );
                        }
                    }
                }
            } else {
                return #ResultFP::Err(
                    #apply_error_::MismatchedKinds {
                        from_kind: #reflect_::reflect_kind(#input_),
                        to_kind: #reflect_kind_::Enum,
                    }
                );
            }
            #ResultFP::Ok(())
        }
    }
}

fn get_enum_to_dynamic_impl(meta: &ReflectMeta) -> TokenStream {
    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let enum_ = crate::path::enum_(vct_reflect_path);

    quote! {
        #[inline]
        fn to_dynamic(&self) -> #alloc_utils_::Box<dyn #reflect_> {
            #alloc_utils_::Box::new( #enum_::to_dynamic_enum(self) )
        }
    }
}

fn get_enum_clone_impl(info: &ReflectEnum) -> TokenStream {
    use crate::path::fp::{ResultFP, CloneFP, OptionFP};

    let meta = info.meta();
    let vct_reflect_path = meta.vct_reflect_path();
    let alloc_utils_ = crate::path::alloc_utils_(vct_reflect_path);
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let reflect_clone_error_ = crate::path::reflect_clone_error_(vct_reflect_path);
    let macro_exports_ = crate::path::macro_exports_(vct_reflect_path);
    let type_path_ = crate::path::type_path_(vct_reflect_path);

    if let Some(span) = meta.attrs().avail_traits.clone {
        quote_spanned! { span =>
            #[inline]
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                #ResultFP::Ok(#alloc_utils_::Box::new(<Self as #CloneFP>::clone(self)) as #alloc_utils_::Box<dyn #reflect_>)
            }
        }
    } else {
        let mut match_tokens = TokenStream::new();

        for variant in info.variants.iter() {
            let ident = &variant.data.ident;
            let variant_name = ident.to_string();
            let variant_path_ = info.variant_path(ident);

            match variant.data.fields {
                syn::Fields::Unit => {
                    match_tokens.extend(quote! {
                        #variant_path_ => #ResultFP::Ok(#alloc_utils_::Box::new(#variant_path_) as #alloc_utils_::Box<dyn #reflect_>),
                    });
                },
                syn::Fields::Named(..) | syn::Fields::Unnamed(..) => {
                    if let Some(ignored_field) = variant.fields().iter().find(|f|f.attrs.ignore.is_some()) {
                        let span = ignored_field.attrs.ignore.unwrap();
                        let field_id = ignored_field.field_id(vct_reflect_path);
                        match_tokens.extend(quote_spanned! { span =>
                            #variant_path_ => #ResultFP::Err(#reflect_clone_error_::FieldNotCloneable {
                                type_path:  #alloc_utils_::Cow::Borrowed(<Self as #type_path_>::type_path())
                                field: #field_id,
                                variant: #OptionFP::Some(#alloc_utils_::Cow::Borrowed(#variant_name)),
                            }),
                        });
                        continue;
                    }
                    let mut member_tokens = TokenStream::new();
                    let mut clone_tokens = TokenStream::new();
                    for (index, field) in variant.fields().iter().enumerate() {
                        let field_ty = &field.data.ty;
                        let member = field.to_member();
                        let accessor = Ident::new(&format!("__mem_{index}"), Span::call_site());

                        member_tokens.extend(quote! {
                            #member: #accessor,
                        });
                        clone_tokens.extend(quote! {
                            #member: #macro_exports_::reflect_clone_field::<#field_ty>(#accessor)?,
                        });
                    }
                    match_tokens.extend(quote! {
                        #variant_path_{ #member_tokens } => #variant_path_ { #clone_tokens },
                    });
                },
            }
        }

        quote! {
            fn reflect_clone(&self) -> #ResultFP<#alloc_utils_::Box<dyn #reflect_>, #reflect_clone_error_> {
                match self {
                    #match_tokens
                }
            }
        }
    }
}

fn get_registry_dependencies(info: &ReflectEnum) -> TokenStream {
    let vct_reflect_path = info.meta().vct_reflect_path();
    let type_registry_ = crate::path::type_registry_(vct_reflect_path);

    let field_types =  info.active_fields().map(|x|&x.data.ty);

    quote! {
        fn register_dependencies(__registry: &mut #type_registry_) {
            #(#type_registry_::register::<#field_types>(__registry);)*
        }
    }
}

fn impl_enum_from_reflect(info: &ReflectEnum) -> TokenStream {
    use crate::path::fp::OptionFP;
    let meta = info.meta();

    let vct_reflect_path = meta.vct_reflect_path();
    let reflect_ = crate::path::reflect_(vct_reflect_path);
    let from_reflect_ = crate::path::from_reflect_(vct_reflect_path);
    let reflect_ref_ = crate::path::reflect_ref_(vct_reflect_path);
    let enum_ = crate::path::enum_(vct_reflect_path);

    let input_ = Ident::new("__input", Span::call_site());

    let clone_tokens = get_common_from_reflect_tokens(meta, &input_);

    // See the `quote!` at the end of the function.
    let mut match_tokens = TokenStream::new();

    for variant in info.variants.iter() {
        let ident = &variant.data.ident;
        let variant_path_ = info.variant_path(ident);
        let variant_name_ = ident.to_string();

        match variant.data.fields {
            syn::Fields::Unit => {
                match_tokens.extend(quote! {
                    #variant_name_ => { return #OptionFP::Some(#variant_path_); },
                });
            },
            syn::Fields::Named(..) | syn::Fields::Unnamed(..) => {
                if variant.fields().iter().any(|f|f.attrs.ignore.is_some()) {
                    // Cannot construct if ignored fields exist.
                    match_tokens.extend(quote! {
                        #variant_name_ => { return #OptionFP::None; },
                    });
                    continue;
                }
                let mut clone_tokens = TokenStream::new();

                for field in variant.fields().iter() {
                    let field_ty = &field.data.ty;
                    let member = field.to_member();

                    let getter = match &field.data.ident {
                        Some(id) => {
                            let name = id.to_string();
                            quote! { #enum_::field(#input_, #name)? }
                        },
                        None => {
                            let index = field.declaration_index; 
                            quote! { #enum_::field_at(#input_, #index)? }
                        },
                    };

                    clone_tokens.extend(quote! {
                        #member: <#field_ty as #from_reflect_>::from_reflect(#getter)?,
                    });
                }

                match_tokens.extend(quote! {
                    #variant_name_ => {
                        let __result = #variant_path_{ #clone_tokens };
                        return #OptionFP::Some(__result);
                    },
                });
            },
        }
    }

    let parser = meta.type_path_parser();
    let real_ident = parser.real_ident();
    let (impl_generics, ty_generics, where_clause) = parser.generics().split_for_impl();

    quote! {
        impl #impl_generics #from_reflect_ for #real_ident #ty_generics #where_clause  {
            fn from_reflect(#input_: &dyn #reflect_) -> #OptionFP<Self> {
                #clone_tokens

                if let #reflect_ref_::Enum(#input_) = #reflect_::reflect_ref(#input_) {
                    match #enum_::variant_name(#input_) {
                        #match_tokens
                        __name => {
                            return #OptionFP::None;
                        }
                    }
                }

                #OptionFP::None
            }
        }
    }
}

