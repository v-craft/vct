use quote::quote;
use syn::{Variant, Ident};
use proc_macro2::Span;
use crate::derive_data::{FieldAttributes, ReflectMeta, StructField};

pub(crate) struct ReflectEnum<'a> {
    pub meta: ReflectMeta<'a>,
    pub variants: Vec<EnumVariant<'a>>,
}

/// Represents a variant on an enum.
pub(crate) struct EnumVariant<'a> {
    /// The raw variant.
    pub data: &'a Variant,
    /// The fields within this variant.
    pub fields: EnumVariantFields<'a>,
    /// The reflection-based attributes on the variant.
    pub attrs: FieldAttributes,
}

pub(crate) enum EnumVariantFields<'a> {
    Named(Vec<StructField<'a>>),
    Unnamed(Vec<StructField<'a>>),
    Unit,
}

impl<'a> EnumVariant<'a> {
    /// The complete set of fields in this variant.
    pub fn fields(&self) -> &[StructField<'a>] {
        match &self.fields {
            EnumVariantFields::Named(fields) | EnumVariantFields::Unnamed(fields) => fields,
            EnumVariantFields::Unit => &[],
        }
    }

    /// Get an iterator of fields which are exposed to the reflection API
    pub fn active_fields(&self) -> impl Iterator<Item = &StructField<'a>> {
        self.fields()
            .iter()
            .filter(|field| !field.attrs.ignore.is_some())
    }

    /// Generates a `TokenStream` for `VariantInfo` construction.
    pub fn to_info_tokens(&self, vct_reflect_path: &syn::Path) -> proc_macro2::TokenStream {
        let variant_info_path = crate::path::variant_info_(vct_reflect_path);

        let variant_info_kind = match &self.fields {
            EnumVariantFields::Named(_) => Ident::new("Struct", Span::call_site()),
            EnumVariantFields::Unnamed(_) => Ident::new("Tuple", Span::call_site()),
            EnumVariantFields::Unit => Ident::new("Unit", Span::call_site()),
        };

        let info_struct_path = match &self.fields {
            EnumVariantFields::Named(_) => crate::path::struct_variant_info_(vct_reflect_path),
            EnumVariantFields::Unnamed(_) => crate::path::tuple_variant_info_(vct_reflect_path),
            EnumVariantFields::Unit => crate::path::unit_variant_info_(vct_reflect_path),
        };

        let fields = self
            .active_fields()
            .map(|field| field.to_info_tokens(vct_reflect_path));

        let variant_name = &self.data.ident.to_string();
        let args = match &self.fields {
            EnumVariantFields::Unit => quote!(#variant_name),
            _ => {
                quote!( #variant_name , &[ #(#fields),* ] )
            }
        };

        // See [`CustomAttributes::get_expression_with`]
        let with_custom_attributes = self.attrs.custom_attributes.get_expression_with(vct_reflect_path);
        // See [`ReflectDocs::get_expression_with`]
        // If feature is diabled, this function will return a empty TokenStream, so it's safe.
        let with_docs = self.attrs.docs.get_expression_with();

        quote! {
            #variant_info_path::#variant_info_kind(
                #info_struct_path::new( #args )
                    #with_custom_attributes
                    #with_docs
            )
        }
    }

}

impl<'a> ReflectEnum<'a> {
    /// Access the metadata associated with this enum definition.
    pub fn meta(&self) -> &ReflectMeta<'a> {
        &self.meta
    }
    /// The complete set of variants in this enum.
    pub fn variants(&self) -> &[EnumVariant<'a>] {
        &self.variants
    }

    /// Get an iterator of fields which are exposed to the reflection API
    pub fn active_fields(&self) -> impl Iterator<Item = &StructField<'a>> {
        self.variants.iter().flat_map(EnumVariant::active_fields)
    }

    /// Returns the given ident as a qualified unit variant of this enum.
    pub fn variant_path(&self, variant: &Ident) -> proc_macro2::TokenStream {
        let name = self.meta.type_path_parser().real_ident();
        quote! {
            #name::#variant
        }
    }

    pub fn to_info_tokens(&self) -> proc_macro2::TokenStream {
        let vct_reflect_path = self.meta.vct_reflect_path();

        let type_info_path = crate::path::type_info_(vct_reflect_path);

        let info_struct_path = crate::path::enum_info_(vct_reflect_path);

        let variant_infos = self.variants.iter()
            .map(|variant| variant.to_info_tokens(vct_reflect_path));

        // See [`CustomAttributes::get_expression_with`]
        let with_custom_attributes = self.meta.with_custom_attributes_expression();
        // See [`ReflectDocs::get_expression_with`]
        // If feature is diabled, this function will return a empty TokenStream, so it's safe.
        let with_docs = self.meta.with_docs_expression();
        // See [`ReflectMeta::with_generics_expression`]
        let with_generics = self.meta.with_generics_expression();

        quote! {
            #type_info_path::Enum(
                #info_struct_path::::new::<Self>(&[ #(#variant_infos),* ])
                    #with_custom_attributes
                    #with_generics
                    #with_docs
            )
        }
    }
}
