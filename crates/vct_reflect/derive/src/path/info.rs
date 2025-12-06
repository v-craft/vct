use proc_macro2::TokenStream;
use quote::quote;

#[inline]
pub(crate) fn type_path_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::TypePath
    }
}

// #[inline(always)]
// pub(crate) fn type_path_table_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::TypePathTable
//     }
// }

#[inline(always)]
pub(crate) fn dynamic_type_path_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::DynamicTypePath
    }
}

// #[inline]
// pub(crate) fn type_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::Type
//     }
// }

#[inline(always)]
pub(crate) fn custom_attributes_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::CustomAttributes
    }
}

#[inline(always)]
pub(crate) fn const_param_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::ConstParamInfo
    }
}

#[inline(always)]
pub(crate) fn generic_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::GenericInfo
    }
}

#[inline(always)]
pub(crate) fn generics_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::Generics
    }
}

#[inline(always)]
pub(crate) fn type_param_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::TypeParamInfo
    }
}

#[inline(always)]
pub(crate) fn field_id_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::FieldId
    }
}

#[inline(always)]
pub(crate) fn named_field_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::NamedField
    }
}

#[inline(always)]
pub(crate) fn unnamed_field_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::UnnamedField
    }
}

#[inline(always)]
pub(crate) fn opaque_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::OpaqueInfo
    }
}

#[inline(always)]
pub(crate) fn struct_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::StructInfo
    }
}

#[inline(always)]
pub(crate) fn tuple_struct_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::TupleStructInfo
    }
}

// #[inline(always)]
// pub(crate) fn tuple_info_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::TupleInfo
//     }
// }

// #[inline(always)]
// pub(crate) fn list_info_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::ListInfo
//     }
// }

// #[inline(always)]
// pub(crate) fn array_info_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::ArrayInfo
//     }
// }

// #[inline(always)]
// pub(crate) fn map_info_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::MapInfo
//     }
// }

// #[inline(always)]
// pub(crate) fn set_info_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::SetInfo
//     }
// }

#[inline(always)]
pub(crate) fn struct_variant_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::StructVariantInfo
    }
}

#[inline(always)]
pub(crate) fn tuple_variant_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::TupleVariantInfo
    }
}

#[inline(always)]
pub(crate) fn unit_variant_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::UnitVariantInfo
    }
}

#[inline(always)]
pub(crate) fn variant_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::VariantInfo
    }
}

#[inline(always)]
pub(crate) fn variant_kind_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::VariantKind
    }
}

#[inline(always)]
pub(crate) fn enum_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::EnumInfo
    }
}

#[inline(always)]
pub(crate) fn reflect_kind_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::ReflectKind
    }
}

// #[inline(always)]
// pub(crate) fn reflect_kind_error_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::ReflectKindError
//     }
// }

#[inline(always)]
pub(crate) fn type_info_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::TypeInfo
    }
}

// #[inline(always)]
// pub(crate) fn dynamic_typed_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::info::DynamicTyped
//     }
// }

#[inline(always)]
pub(crate) fn typed_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::info::Typed
    }
}
