use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{spanned::Spanned, LitStr};

/// An enum representing different types of string expressions
#[derive(Clone)]
pub(crate) enum StringExpr {
    /// A string that is valid at compile time.
    /// 
    /// In most cases, this is a string lit, such as: `"mystring"`.
    /// 
    /// But sometimes, this also includes macros, such as: `module_path!(xxx)`
    Const(TokenStream),
    /// A [string slice](str) that is borrowed for a `'static` lifetime.
    /// 
    /// For example: `a`, a is a `&'static str`.
    Borrowed(TokenStream),
    /// An [owned string](String).
    /// 
    /// For example: `a`, a is a [`String`].
    Owned(TokenStream),
}

impl StringExpr {
    /// Creates a [constant] [`StringExpr`] from a [`struct@LitStr`].
    ///
    /// [constant]: StringExpr::Const
    pub fn from_lit(lit: &LitStr) -> Self {
        Self::Const(lit.to_token_stream())
    }

    /// Creates a [constant] [`StringExpr`] by interpreting a [string slice][str] as a [`struct@LitStr`].
    ///
    /// [constant]: StringExpr::Const
    pub fn from_str(string: &str) -> Self {
        // ↓ Generate tokens with string literal.
        Self::Const(string.into_token_stream())
    }

    /// Returns tokens for a statically borrowed [string slice](str).
    pub fn into_borrowed(self) -> TokenStream {
        match self {
            Self::Const(tokens) | Self::Borrowed(tokens) => tokens,
            Self::Owned(owned) => quote! {
                &#owned
            },
        }
    }

    /// Returns tokens for an [owned string](String).
    pub fn into_owned(self, vct_reflect_path: &syn::Path) -> TokenStream {
        let alloc_utils_path = crate::path::alloc_utils_(&vct_reflect_path);

        match self {
            Self::Const(tokens) | Self::Borrowed(tokens) => quote! {
                #alloc_utils_path::ToString::to_string(#tokens)
            },
            Self::Owned(owned) => owned,
        }
    }


    /// Concat two string expr.
    ///
    /// If both expressions are [`StringExpr::Const`] this will use [`concat`] to merge them.
    pub fn concat(self, other: StringExpr, vct_reflect_path: &syn::Path) -> Self {
        if let Self::Const(tokens) = &self {
            if let Self::Const(more) = other {
                return Self::Const(quote! {
                    ::core::concat!(#tokens, #more)
                });
            }
        }

        let owned = self.into_owned(vct_reflect_path);
        let borrowed = other.into_borrowed();
        Self::Owned(quote! {
            ::core::ops::Add::<&str>::add(#owned, #borrowed)
        })
    }


    pub fn from_iter<T: IntoIterator<Item = StringExpr>>(iter: T, vct_reflect_path: &syn::Path) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            Some(mut expr) => {
                for next in iter {
                    expr = expr.concat(next, vct_reflect_path);
                }
                expr
            }
            None => Default::default(),
        }
    }

}

impl<T: ToString + Spanned> From<T> for StringExpr {
    fn from(value: T) -> Self {
        Self::from_lit(&LitStr::new(&value.to_string(), value.span()))
    }
}

impl Default for StringExpr {
    fn default() -> Self {
        StringExpr::from_str("")
    }
}

// impl FromIterator<StringExpr> for StringExpr {
//     fn from_iter<T: IntoIterator<Item = StringExpr>>(iter: T) -> Self {
//         let mut iter = iter.into_iter();
//         match iter.next() {
//             Some(mut expr) => {
//                 let vct_reflect_path = crate::path::vct_reflect();
//                 for next in iter {
//                     expr = expr.concat(next, &vct_reflect_path);
//                 }
//                 expr
//             }
//             None => Default::default(),
//         }
//     }
// }


