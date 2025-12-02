use proc_macro2::Span;
use syn::{Attribute, Path, Meta, Token, MacroDelimiter, MetaList, parse::ParseStream, MetaNameValue, spanned::Spanned};

use crate::{
    REFLECT_ATTRIBUTE_NAME,
    derive_data::{CustomAttributes, ReflectDocs}
};

mod kw{
    syn::custom_keyword!(docs);
    syn::custom_keyword!(ignore);
    syn::custom_keyword!(skip_serde);
}

/// Stores data about if the field should be visible via the Reflect and serialization interfaces
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldIgnoreKind {
    /// Don't ignore, appear to all systems
    #[default]
    None,
    /// Ignore when serializing but not when reflecting
    Serde,
    /// Ignore both when serializing and reflecting
    All,
}

impl FieldIgnoreKind {
    pub fn is_active(self) -> bool {
        self != FieldIgnoreKind::All
    }

    pub fn is_ignored(self) -> bool {
        self == FieldIgnoreKind::All
    }
}

/// Controls how the default value is determined for a field.
#[derive(Default, Clone)]
pub(crate) enum FieldDefaultKind {
    /// Field is required.
    #[default]
    None,
    /// Field can be defaulted using `Default::default()`.
    Trait,
    /// Field can be created using the given function name.
    ///
    /// This assumes the function is in scope, is callable with zero arguments,
    /// and returns the expected type.
    Custom(Path, Span),
}

#[derive(Default, Clone)]
pub(crate) struct FieldAttributes {
    /// Custom attributes created via `#[reflect(@...)]`.
    pub custom_attributes: CustomAttributes,
    /// Custom docs: `///`, `#[doc = ""]` or `#[reflect(docs = "")]`
    pub docs: ReflectDocs,
    /// Determines how this field should be ignored if at all.
    pub ignore: FieldIgnoreKind,
    /// Sets the default behavior of this field.
    pub default: FieldDefaultKind,
}

impl FieldAttributes {
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut field_attributes = FieldAttributes::default();

        for attribute in attrs {
            match &attribute.meta {
                Meta::List(meta_list) if meta_list.path.is_ident(REFLECT_ATTRIBUTE_NAME) => {
                    if let MacroDelimiter::Paren(_) = meta_list.delimiter {
                        // ↑ Muse use `()` in `#[reflect(...)]`, instead of `{...}` or `[...]`.
                        field_attributes.parse_meta_list(meta_list)?;
                    } else {
                        return Err(syn::Error::new(
                            meta_list.delimiter.span().join(),
                            format_args!(
                                "`#[{REFLECT_ATTRIBUTE_NAME}(\"...\")]` must use parentheses `(` and `)`"
                            ),
                        ));
                    }
                },
                #[cfg(feature = "reflect_docs")]
                Meta::NameValue(pair) if pair.path.is_ident("doc") => {
                    field_attributes.docs.parse_default_docs(pair)?;
                }
                _ => continue,
            }
        }

        Ok(field_attributes)
    }

    fn parse_meta_list(&mut self, meta: &MetaList) -> syn::Result<()> {
        meta.parse_args_with(|stream: ParseStream|{
            loop {
                if stream.is_empty() {
                    break;
                }
                self.parse_inner_attribute(stream)?;
                if stream.is_empty() {
                    break;
                }
                stream.parse::<Token![,]>()?;
            }
            Ok(())
        })
    }


    fn parse_inner_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![@]) {
            self.parse_custom_attribute(input)
        } else if lookahead.peek(kw::docs) {
            self.parse_docs(input)
        } else {
            Err(lookahead.error())
        }
    }

    fn parse_custom_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        self.custom_attributes.parse_inner_stream(input)
    }

    /// This function can be used when the `reflect_docs` feature is disabled.
    /// When the feature is not enabled, it will not do anything.
    fn parse_docs(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(docs = "...")]
        let pair = input.parse::<MetaNameValue>()?;
        self.docs.parse_custom_docs(&pair)
    }

    fn parse_ignore(&mut self, input: ParseStream) -> syn::Result<()> {
        if self.ignore != FieldIgnoreKind::None {
            return Err(input.error("Only one of [`ignore`, `skip_serde`] is allowed."))
        }
        input.parse::<kw::ignore>()?;
        self.ignore = FieldIgnoreKind::All;
        Ok(())
    }

    fn parse_skip_serde(&mut self, input: ParseStream) -> syn::Result<()> {
        if self.ignore != FieldIgnoreKind::None {
            return Err(input.error("Only one of [`ignore`, `skip_serde`] is allowed."))
        }
        input.parse::<kw::ignore>()?;
        self.ignore = FieldIgnoreKind::Serde;
        Ok(())
    }

}




