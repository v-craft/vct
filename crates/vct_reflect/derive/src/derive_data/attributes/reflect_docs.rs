//! Contains code related to documentation reflection (requires the `documentation` feature).

use crate::path::fp::OptionFP;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Expr, ExprLit, Lit, MetaNameValue, spanned::Spanned};


/// A struct used to represent a type's documentation, if any.
///
/// When converted to a [`TokenStream`], this will output an `Option<String>`
/// containing the collection of doc comments.
#[derive(Clone)]
pub(crate) struct ReflectDocs {
    enabled: bool,
    is_custom: bool,
    docs: Vec<String>,   // `#[reflect(docs = "...")]`
}

impl Default for ReflectDocs {
    fn default() -> Self {
        Self {
            #[cfg(feature = "reflect_docs")]
            enabled: true,
            #[cfg(not(feature = "reflect_docs"))]
            enabled: false,
            is_custom: false,
            docs: Vec::new(),
        }
    }
}

impl ReflectDocs {
    /// Is the collection empty?
    pub fn is_empty(&self) -> bool {
        !self.enabled || self.docs.is_empty()
    }

    /// Parse reflect attribute docs.
    /// 
    /// This function do **not** check if the key is `docs`, 
    /// it is guaranteed by the caller.
    ///
    /// Examples:
    /// - `#[doc = "..."]`
    pub fn parse_default_docs(&mut self, pair: &MetaNameValue) -> syn::Result<()> {
        if self.enabled && !self.is_custom {
            if let Expr::Lit(ExprLit {
                lit: Lit::Str(lit_str), ..
            }) = &pair.value {
                self.docs.push(lit_str.value());
            } else {
                return Err(syn::Error::new(pair.value.span(), "`#[doc = ...]` expected a string literal value"));
            }
        }
        Ok(())
    }

    /// Parse reflect attribute docs.
    /// 
    /// This function do **not** check if the key is `docs`, 
    /// it is guaranteed by the caller.
    ///
    /// Examples:
    /// - `#[reflect(docs = "...")]`
    pub fn parse_custom_docs(&mut self, pair: &MetaNameValue) -> syn::Result<()> {
        if let Expr::Lit(expr_lit) = &pair.value {
            match &expr_lit.lit {
                Lit::Str(lit_str) => {
                    if self.enabled { // Check inside to avoid ignoring syntax check when enable false.
                        if !self.is_custom {
                            self.docs.clear();
                            self.is_custom = true;
                        }
                        self.docs.push(lit_str.value());
                    }
                },
                Lit::Bool(lit_bool) => {
                    if lit_bool.value() {
                        return Err(syn::Error::new(expr_lit.span(), "Explicit `true` is invalid, it's default value if `reflect_docs` is enabled."));
                    }
                    if self.enabled {
                        self.enabled = false;
                        self.docs.clear();
                    }
                },
                _ => return Err(syn::Error::new(expr_lit.span(), "Expected a string or `false` literal")),
            }
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a string or `false` literal"));
        }
        
        
        Ok(())
    }

    pub fn doc_string(&self) -> Option<String> {
        if !self.enabled || self.docs.is_empty() {
            return None;
        }

        let len = self.docs.len();
        let capacity = self.docs.iter().map(String::len).sum::<usize>() + len;
        if capacity == len {
            return None; // Empty document
        }

        let mut res = String::with_capacity(capacity);
        for s in &self.docs {
            res.push_str(s);
            res.push('\n');
        }
        res.pop(); // delete the last `\n`

        Some(res)
    }
}

impl ToTokens for ReflectDocs {
    fn to_tokens(&self, _tokens: &mut TokenStream) {
        if let Some(doc) = self.doc_string() {
            quote!(#OptionFP::Some(#doc)).to_tokens(_tokens);
        } else {
            quote!(#OptionFP::None).to_tokens(_tokens);
        }
    }
}

