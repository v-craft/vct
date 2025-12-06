use proc_macro2::TokenStream;
use quote::quote;

#[inline]
pub(crate) fn apply_error_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::ApplyError
    }
}

#[inline]
pub(crate) fn reflect_clone_error_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::ReflectCloneError
    }
}

#[inline]
pub(crate) fn reflect_mut_(vct_reflect_path: &syn::Path) -> TokenStream {  
    quote! {
        #vct_reflect_path::ops::ReflectMut
    }
}
    
#[inline]
pub(crate) fn reflect_owned_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::ReflectOwned
    }
}
    
#[inline]
pub(crate) fn reflect_ref_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::ReflectRef
    }
}

#[inline]
pub(crate) fn dynamic_struct_(vct_reflect_path: &syn::Path) -> TokenStream {  
    quote! {
        #vct_reflect_path::ops::DynamicStruct
    }
}
    
// #[inline]
// pub(crate) fn get_struct_field_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::GetStructField
//     }
// }
    
#[inline]
pub(crate) fn struct_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::Struct
    }
}

#[inline]
pub(crate) fn struct_field_iter_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::StructFieldIter
    }
}

// #[inline]
// pub(crate) fn struct_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::struct_debug
//     }
// }

// #[inline]
// pub(crate) fn struct_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::struct_partial_eq
//     }
// }

#[inline]
pub(crate) fn dynamic_tuple_struct_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::DynamicTupleStruct
    }
}

// #[inline]
// pub(crate) fn get_tuple_struct_field_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::GetTupleStructField
//     }
// }

#[inline]
pub(crate) fn tuple_struct_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::TupleStruct
    }
}

#[inline]
pub(crate) fn tuple_struct_field_iter_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::TupleStructFieldIter
    }
}

// #[inline]
// pub(crate) fn tuple_struct_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::tuple_struct_debug
//     }
// }

// #[inline]
// pub(crate) fn tuple_struct_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::tuple_struct_partial_eq
//     }
// }

// #[inline]
// pub(crate) fn dynamic_tuple_(vct_reflect_path: &syn::Path) -> TokenStream {  
//     quote! {
//         #vct_reflect_path::ops::DynamicTuple
//     }
// }
    
// #[inline]
// pub(crate) fn get_tuple_field_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::GetTupleField
//     }
// }
    
// #[inline]
// pub(crate) fn tuple_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::Tuple
//     }
// }

// #[inline]
// pub(crate) fn tuple_field_iter_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::TupleFieldIter
//     }
// }

// #[inline]
// pub(crate) fn tuple_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::tuple_debug
//     }
// }

// #[inline]
// pub(crate) fn tuple_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::tuple_partial_eq
//     }
// }

// #[inline]
// pub(crate) fn tuple_try_apply_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::tuple_try_apply
//     }
// }

// #[inline]
// pub(crate) fn dynamic_list_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::DynamicList
//     }
// }
    
// #[inline]
// pub(crate) fn list_(vct_reflect_path: &syn::Path) -> TokenStream {        
//     quote! {
//         #vct_reflect_path::ops::List
//     }
// }
    
// #[inline]
// pub(crate) fn list_item_iter_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::ListItemIter
//     }
// }

// #[inline]
// pub(crate) fn list_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::list_debug
//     }
// }

// #[inline]
// pub(crate) fn list_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::list_partial_eq
//     }
// }

// #[inline]
// pub(crate) fn array_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::Array
//     }
// }
    
// #[inline]
// pub(crate) fn array_item_iter_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::ArrayItemIter
//     }
// }
    
// #[inline]
// pub(crate) fn dynamic_array_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::DynamicArray
//     }
// }

// #[inline]
// pub(crate) fn array_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::array_debug
//     }
// }

// #[inline]
// pub(crate) fn array_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::array_partial_eq
//     }
// }

// #[inline]
// pub(crate) fn dynamic_map_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::DynamicMap
//     }
// }
    
// #[inline]
// pub(crate) fn map_(vct_reflect_path: &syn::Path) -> TokenStream {        
//     quote! {
//         #vct_reflect_path::ops::Map
//     }
// }
    
// #[inline]
// pub(crate) fn map_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::map_debug
//     }
// }

// #[inline]
// pub(crate) fn map_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::map_partial_eq
//     }
// }

// #[inline]
// pub(crate) fn dynamic_set_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::DynamicSet
//     }
// }
    
// #[inline]
// pub(crate) fn set_(vct_reflect_path: &syn::Path) -> TokenStream {        
//     quote! {
//         #vct_reflect_path::ops::Set
//     }
// }
    
// #[inline]
// pub(crate) fn set_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::set_debug
//     }
// }

// #[inline]
// pub(crate) fn set_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::set_partial_eq
//     }
// }

// #[inline]
// pub(crate) fn dynamic_variant_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::DynamicVariant
//     }
// }
    
// #[inline]
// pub(crate) fn variant_field_(vct_reflect_path: &syn::Path) -> TokenStream {  
//     quote! {
//         #vct_reflect_path::ops::VariantField
//     }
// }
    
#[inline]
pub(crate) fn variant_field_iter_(vct_reflect_path: &syn::Path) -> TokenStream {
    quote! {
        #vct_reflect_path::ops::VariantFieldIter
    }
}

// #[inline]
// pub(crate) fn dynamic_enum_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::DynamicEnum
//     }
// }
    
#[inline]
pub(crate) fn enum_(vct_reflect_path: &syn::Path) -> TokenStream {        
    quote! {
        #vct_reflect_path::ops::Enum
    }
}
    
// #[inline]
// pub(crate) fn enum_debug_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::enum_debug
//     }
// }

// #[inline]
// pub(crate) fn enum_partial_eq_(vct_reflect_path: &syn::Path) -> TokenStream {
//     quote! {
//         #vct_reflect_path::ops::enum_partial_eq
//     }
// }
