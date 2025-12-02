use syn::{DeriveInput, spanned::Spanned};

use crate::{ImplSourceKind, derive_data::{ReflectEnum, ReflectMeta, ReflectStruct, ReflectTypePath, TypeAttributes}};

pub(crate) enum ReflectDerive<'a> {
    Struct(ReflectStruct<'a>),
    TupleStruct(ReflectStruct<'a>),
    UnitStruct(ReflectStruct<'a>),
    Enum(ReflectEnum<'a>),
    Opaque(ReflectMeta<'a>),
}


impl<'a> ReflectDerive<'a> {
    pub fn from_input(input: &'a DeriveInput, source: ImplSourceKind) -> syn::Result<Self> {
        let type_attributes = TypeAttributes::parse_attrs(&input.attrs)?;

        // For local types, can use `module_path!()` to get the module path, 
        // but for foreign types, the user needs to explicitly provide it.
        // If automatic implementation is disabled, it can also be ignored.
        if source == ImplSourceKind::ImplForeignType
            && type_attributes.trait_flags.impl_type_path
            && type_attributes.type_path.is_none()
        {
            return Err(syn::Error::new(
                input.ident.span(), 
                "#[reflect(type_path = \"...\")] must be specified when auto impl TypePath for Foreign Type.",
            ));
        }

        // After meeting the above conditions, they can all be considered as local types.
        let type_path = ReflectTypePath::Local { 
            ident: &input.ident,
            custom_path: type_attributes.type_path.clone(),
            generics: &input.generics
        };

        let meta = ReflectMeta::new(type_attributes, type_path);

        if meta.attrs().is_opaque {
            return Ok(Self::Opaque(meta));
        }

        match &input.data {
            syn::Data::Struct(data_struct) => {
                todo!()
            },
            syn::Data::Enum(data_enum) => {
                todo!()
            },
            syn::Data::Union(_) => {
                Err(syn::Error::new(input.span(), "reflection macros do not support unions."))
            },
        }
    }
}
