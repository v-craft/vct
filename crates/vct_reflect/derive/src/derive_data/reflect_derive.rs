use syn::{DeriveInput, token::Comma, Fields, punctuated::Punctuated, spanned::Spanned, Variant};

use crate::{
    ImplSourceKind, 
    derive_data::{
        EnumVariant, EnumVariantFields, FieldAttributes, ReflectEnum, ReflectMeta, ReflectStruct, TypePathParser, StructField, TypeAttributes
    }
};

pub(crate) enum ReflectDerive<'a> {
    Struct(ReflectStruct<'a>),
    TupleStruct(ReflectStruct<'a>),
    UnitStruct(ReflectMeta<'a>),
    Enum(ReflectEnum<'a>),
    Opaque(ReflectMeta<'a>),
}


impl<'a> ReflectDerive<'a> {
    // pub fn meta(&self) -> &ReflectMeta<'a> {
    //     match self {
    //         ReflectDerive::Struct(reflect_struct) => reflect_struct.meta(),
    //         ReflectDerive::TupleStruct(reflect_struct) => reflect_struct.meta(),
    //         ReflectDerive::UnitStruct(reflect_meta) => reflect_meta,
    //         ReflectDerive::Enum(reflect_enum) => reflect_enum.meta(),
    //         ReflectDerive::Opaque(reflect_meta) => reflect_meta,
    //     }
    // }

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
        //
        // There are other algorithms for Foreign TypePath and Primitive TypePath.
        let type_path = TypePathParser::Local { 
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
                let fields = Self::colloct_struct_field(&data_struct.fields)?;

                match data_struct.fields {
                    Fields::Named(..) => Ok(Self::Struct(ReflectStruct { meta, fields })),
                    Fields::Unnamed(..) => Ok(Self::TupleStruct(ReflectStruct { meta, fields })),
                    Fields::Unit => Ok(Self::UnitStruct(meta)),
                }
            },
            syn::Data::Enum(data_enum) => {
                let variants = Self::collect_enum_variants(&data_enum.variants)?;
                Ok(Self::Enum(ReflectEnum { meta, variants }))
            },
            syn::Data::Union(_) => {
                Err(syn::Error::new(input.span(), "reflection macros do not support unions."))
            },
        }
    }

    fn colloct_struct_field(fields: &'a Fields) -> syn::Result<Vec<StructField<'a>>> {
        let mut active_index = 0;
        
        let mut res: Vec<StructField<'a>> = Vec::with_capacity(fields.len());

        for (declaration_index, field) in fields.iter().enumerate() {
            let attrs = FieldAttributes::parse_attrs(&field.attrs)?;

            let reflection_index = if attrs.ignore {
                None
            } else {
                active_index += 1;
                Some(active_index - 1)
            };

            res.push(StructField {
                data: field,
                attrs,
                declaration_index,
                reflection_index
            });
        }

        Ok(res)
    }

    fn collect_enum_variants(variants: &'a Punctuated<Variant, Comma>) -> syn::Result<Vec<EnumVariant<'a>>> {
        let mut res: Vec<EnumVariant<'a>> = Vec::with_capacity(variants.len());

        for variant in variants.iter() {
            let fields = Self::colloct_struct_field(&variant.fields)?;
            let variant_fields = match variant.fields {
                Fields::Named(..) => EnumVariantFields::Named(fields),
                Fields::Unnamed(..) => EnumVariantFields::Unnamed(fields),
                Fields::Unit => EnumVariantFields::Unit,
            };
            res.push(EnumVariant { 
                data: variant, 
                fields: variant_fields, 
                attrs: FieldAttributes::parse_attrs(&variant.attrs)?,
            });
        }

        Ok(res)
    }
    
}
