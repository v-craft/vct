use syn::{
    Attribute, Expr, ExprLit, ExprPath, Lit, MacroDelimiter, Meta, MetaList, MetaNameValue, Path, Token, TypePath, parse::ParseStream, spanned::Spanned
};

use crate::{
    REFLECT_ATTRIBUTE_NAME,
    derive_data::{
        CustomAttributes, MethodFlag, MethodImplFlags, ReflectDocs, TraitImplFlags
    }
};

mod kw {
    syn::custom_keyword!(TypePath);
    syn::custom_keyword!(Typed);
    syn::custom_keyword!(Reflect);
    syn::custom_keyword!(GetTypeTraits);
    syn::custom_keyword!(FromReflect);
    syn::custom_keyword!(Struct);
    syn::custom_keyword!(TupleStruct);
    syn::custom_keyword!(Tuple);
    syn::custom_keyword!(Enum);
    syn::custom_keyword!(opaque);
    syn::custom_keyword!(clone);
    syn::custom_keyword!(Clone);    
    syn::custom_keyword!(debug);
    syn::custom_keyword!(Debug);
    syn::custom_keyword!(hash);
    syn::custom_keyword!(Hash);
    syn::custom_keyword!(partial_eq);
    syn::custom_keyword!(PartialEq);
    syn::custom_keyword!(default);
    syn::custom_keyword!(Default);
    syn::custom_keyword!(Internal);
    syn::custom_keyword!(type_path);
    syn::custom_keyword!(auto_register);
    syn::custom_keyword!(docs);
    syn::custom_keyword!(alias);
}

#[derive(Default, Clone)]
pub(crate) struct TypeAttributes {
    /// See: [`CustomAttributes`]
    pub custom_attributes: CustomAttributes,
    /// See: [`TraitImplFlags`]
    pub trait_flags: TraitImplFlags,
    /// See: [`MethodImplFlags`]
    pub method_flags: MethodImplFlags,
    /// By default, only types like `struct A;` are `Opaque`, but user can use `#[reflect(Opaque)]` to enable it explicitly.
    pub is_opaque: bool,
    /// Default is false, use `#[reflect(auto_register)]` or `#[reflect(auto_register = true)]` to enable i.
    pub auto_register: bool,
    /// Default is None, So the macro will be auto generated. Use `#[reflect(type_path = "...")]` to set it.
    pub type_path: Option<Path>,
    /// Default is Empty Docs,  Use `///`, `#[doc = ""]` or `#[reflect(docs = "")]` to set it, Can set multi-lines.
    pub docs: ReflectDocs,
    /// Default is None,  Use `#[reflect(alias = ...)]` to set it.
    pub alias: Option<TypePath>,
}

impl TypeAttributes {
    /// try parse [`TypeAttributes`] from [`syn::Attribute`]
    pub fn parse_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut type_attributes = TypeAttributes::default();

        for attribute in attrs {
            match &attribute.meta {
                Meta::List(meta_list) if meta_list.path.is_ident(REFLECT_ATTRIBUTE_NAME) => {
                    // ↑ Must be `#[reflect(...)]` instead of `#[key = val]` or `#[attr]`.
                    if let MacroDelimiter::Paren(_) = meta_list.delimiter {
                        // ↑ Muse use `()` in `#[reflect(...)]`, instead of `{...}` or `[...]`.
                        type_attributes.parse_meta_list(meta_list)?;
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
                    type_attributes.docs.parse_default_docs(pair)?;
                }
                _ => continue,
            }
        }

        Ok(type_attributes)
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
        } else if lookahead.peek(kw::type_path) {
            self.parse_type_path(input)
        } else if lookahead.peek(kw::clone) {
            self.parse_reflect_clone(input)
        } else if lookahead.peek(kw::debug) {
            self.parse_reflect_debug(input)
        } else if lookahead.peek(kw::hash) {
            self.parse_reflect_hash(input)
        } else if lookahead.peek(kw::partial_eq) {
            self.parse_reflect_patrial_eq(input)
        } else if lookahead.peek(kw::default){
            self.parse_reflect_default(input)
        } else if lookahead.peek(kw::opaque) {
            self.parse_opaque(input)
        } else if lookahead.peek(kw::auto_register) {
            self.parse_auto_register(input)
        } else if lookahead.peek(kw::TypePath) {
            self.parse_trait_type_path(input)
        } else if lookahead.peek(kw::Typed) {
            self.parse_trait_typed(input)
        } else if lookahead.peek(kw::Reflect) {
            self.parse_trait_reflect(input)
        } else if lookahead.peek(kw::GetTypeTraits) {
            self.parse_trait_get_type_traits(input)
        } else if lookahead.peek(kw::FromReflect) {
            self.parse_trait_from_reflect(input)
        } else if lookahead.peek(kw::Struct) {
            self.parse_trait_struct(input)
        } else if lookahead.peek(kw::TupleStruct) {
            self.parse_trait_tuple_struct(input)
        } else if lookahead.peek(kw::Tuple) {
            self.parse_trait_tuple(input)
        } else if lookahead.peek(kw::Enum) {
            self.parse_trait_enum(input)
        } else if lookahead.peek(kw::alias) {
            self.parse_alias(input)
        } else {
            Err(lookahead.error())
        }
    }

    fn parse_custom_attribute(&mut self, input: ParseStream) -> syn::Result<()> {
        self.custom_attributes.parse_inner_stream(input)
    }

    fn parse_trait_type_path(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(TypePath = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_type_path = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_typed(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(Typed = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_typed = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_reflect(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(Reflecct = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_reflect = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_get_type_traits(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(GetTypeTraits = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_get_type_traits = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_from_reflect(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(FromReflect = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_from_reflect = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_struct(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(Struct = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_struct = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_tuple_struct(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(TupleStruct = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_tuple_struct = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_trait_tuple(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(Tuple = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_tuple = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        
        Ok(())
    }

    fn parse_trait_enum(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(Enum = false)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Bool(lit), ..
        }) = &pair.value {
            if lit.value() {
                return Err(syn::Error::new(lit.span(), "Should not be `true`, it's default value."));
            }
            self.trait_flags.impl_enum = lit.value();
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a bool value."));
        }
        
        Ok(())
    }

    fn parse_reflect_clone(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(clone = Clone/Internal/func_path)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Path(ExprPath{ path, .. }) = &pair.value {
            if path.is_ident("Internal") {
                self.method_flags.reflect_clone = MethodFlag::Internal(path.span());
            } else if path.is_ident("Clone") {
                self.method_flags.reflect_clone = MethodFlag::Trait(path.span());
            } else {
                self.method_flags.reflect_clone = MethodFlag::Custom(path.clone(), path.span());
            }
        } else {
            return Err(syn::Error::new(pair.value.span(), "Epected a Path."));
        }
        
        Ok(())
    }

    fn parse_reflect_hash(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(hash = Hash/Internal/func_path)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Path(ExprPath{ path, .. }) = &pair.value {
            if path.is_ident("Internal") {
                self.method_flags.reflect_hash = MethodFlag::Internal(path.span());
            } else if path.is_ident("Hash") {
                self.method_flags.reflect_hash = MethodFlag::Trait(path.span());
            } else {
                self.method_flags.reflect_hash = MethodFlag::Custom(path.clone(), path.span());
            }
        } else {
            return Err(syn::Error::new(pair.value.span(), "Epected a Path."));
        }
        
        Ok(())
    }

    fn parse_reflect_debug(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(debug = Debug/Internal/func_path)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Path(ExprPath{ path, .. }) = &pair.value {
            if path.is_ident("Internal") {
                self.method_flags.reflect_debug = MethodFlag::Internal(path.span());
            } else if path.is_ident("Debug") {
                self.method_flags.reflect_debug = MethodFlag::Trait(path.span());
            } else {
                self.method_flags.reflect_debug = MethodFlag::Custom(path.clone(), path.span());
            }
        } else {
            return Err(syn::Error::new(pair.value.span(), "Epected a Path."));
        }
        
        Ok(())
    }

    fn parse_reflect_patrial_eq(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(partial_eq = PartialEq/Internal/func_path)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Path(ExprPath{ path, .. }) = &pair.value {
            if path.is_ident("Internal") {
                self.method_flags.reflect_partial_eq = MethodFlag::Internal(path.span());
            } else if path.is_ident("PartialEq") {
                self.method_flags.reflect_partial_eq = MethodFlag::Trait(path.span());
            } else {
                self.method_flags.reflect_partial_eq = MethodFlag::Custom(path.clone(), path.span());
            }
        } else {
            return Err(syn::Error::new(pair.value.span(), "Epected a Path."));
        }
        
        Ok(())
    }

    fn parse_reflect_default(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(default = Default/Internal/func_path)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Path(ExprPath{ path, .. }) = &pair.value {
            if path.is_ident("Internal") {
                self.method_flags.reflect_default = MethodFlag::Internal(path.span());
            } else if path.is_ident("Default") {
                self.method_flags.reflect_default = MethodFlag::Trait(path.span());
            } else {
                self.method_flags.reflect_default = MethodFlag::Custom(path.clone(), path.span());
            }
        } else {
            return Err(syn::Error::new(pair.value.span(), "Epected a Path."));
        }
        
        Ok(())
    }

    fn parse_opaque(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<kw::opaque>()?;
        self.is_opaque = true;
        Ok(())
    }

    fn parse_auto_register(&mut self, input: ParseStream) -> syn::Result<()> {
        input.parse::<kw::auto_register>()?;
        self.auto_register = true;
        Ok(())
    }

    fn parse_type_path(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(type_path = "...")]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Lit(ExprLit {
            lit: Lit::Str(lit), ..
        }) = &pair.value {
            let path: Path = syn::parse_str(&lit.value())?;
            if path.segments.is_empty() {
                return Err(syn::Error::new(lit.span(), "`type_path` should not be empty."));
            }
            if path.leading_colon.is_some() {
                return Err(syn::Error::new(lit.span(), "`type_path` should not have leading-colon."));
            }
            self.type_path = Some(path);
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a string liternal value."));
        }
        
        Ok(())
    }

    /// This function can be used when the `reflect_docs` feature is disabled.
    /// When the feature is not enabled, it will not do anything.
    fn parse_docs(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(docs = "...")]
        let pair = input.parse::<MetaNameValue>()?;
        self.docs.parse_custom_docs(&pair)
    }

    fn parse_alias(&mut self, input: ParseStream) -> syn::Result<()> {
        // #[reflect(alias = ...)]
        let pair = input.parse::<MetaNameValue>()?;

        if let Expr::Path(ExprPath{path, ..}) = &pair.value {
            self.alias =  Some(TypePath {
                qself: None,
                path: path.clone(),
            });
        } else {
            return Err(syn::Error::new(pair.value.span(), "Expected a type path."));
        }
        
        Ok(())
    }

}













