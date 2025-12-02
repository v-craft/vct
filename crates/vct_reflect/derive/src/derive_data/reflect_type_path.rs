use quote::{ToTokens, quote};
use syn::{Ident, Path, Generics, punctuated::Punctuated, GenericParam, spanned::Spanned, LitStr, TypeParam};
use crate::utils::StringExpr;


pub(crate) enum ReflectTypePath<'a> {
    /// Types without a crate/module that can be named from any scope (e.g. `bool`).
    Primitive(&'a Ident),
    /// The name of a type relative to its scope.
    ///
    /// The type must be able to be reached with just its name.
    /// 
    /// For local types, can use [`module_path!()`](module_path) to get the module path.
    Local {
        ident: &'a Ident,
        custom_path: Option<Path>,
        generics: &'a Generics,
    },
    /// Using `::my_crate::foo::Bar` syntax.
    ///
    /// May have a separate custom path used for the `TypePath` implementation.
    Foreign {
        path: &'a Path,
        custom_path: Option<Path>,
        generics: &'a Generics,
    },
}

impl<'a> ReflectTypePath<'a> {
    pub fn has_custom_path(&self) -> bool {
        match self {
            Self::Local { custom_path, .. } | Self::Foreign { custom_path, .. } => {
                custom_path.is_some()
            }
            _ => false,
        }
    }

    pub fn generics(&self) -> &'a Generics {
        // Use a constant because we need to return a reference of at least 'a.
        const EMPTY_GENERICS: &Generics = &Generics {
            gt_token: None,
            lt_token: None,
            where_clause: None,
            params: Punctuated::new(),
        };

        match self {
            Self::Local { generics, .. } | Self::Foreign { generics, .. } => generics,
            _ => EMPTY_GENERICS,
        }
    }

    /// Whether an implementation of `Typed` or `TypePath` should be generic.
    pub fn impl_with_generic(&self) -> bool {
        !self
            .generics()
            .params
            .iter()
            .all(|param| matches!(param, GenericParam::Lifetime(_)))
    }

    pub fn get_ident(&self) -> Option<&Ident> {
        match self {
            Self::Primitive(ident) => Some(ident),
            Self::Local{ ident, custom_path, .. } => Some(
                custom_path
                    .as_ref()
                    .map(|path| &path.segments.last().unwrap().ident)
                    .unwrap_or(ident),
            ),
            Self::Foreign{ path, custom_path, .. } => Some(
                &custom_path
                    .as_ref()
                    .unwrap_or(path)
                    .segments
                    .last()
                    .unwrap()
                    .ident,
            ),
        }
    }

    pub fn get_path(&self) -> Option<&Path> {
        match self {
            Self::Local{ custom_path, .. } => custom_path.as_ref(),
            Self::Foreign{ path, custom_path, .. } => Some(
                custom_path.as_ref().unwrap_or(path)
            ),
            _ => None,
        }
    }

    pub fn true_type(&self) -> proc_macro2::TokenStream {
        match self {
            Self::Primitive(ident) => quote!(#ident),
            Self::Local{ ident, generics, .. } => {
                let (_, ty_generics, _) = generics.split_for_impl();
                quote!(#ident #ty_generics)
            }
            Self::Foreign{ path, generics, .. } => {
                let (_, ty_generics, _) = generics.split_for_impl();
                quote!(#path #ty_generics)
            }
        }
    }

    pub fn crate_name(&self) -> Option<StringExpr> {
        if let Some(path) = self.get_path() {
            let crate_name = &path.segments.first().unwrap().ident;
            return Some(StringExpr::from(crate_name));
        }

        match self {
            Self::Local { .. } => Some(StringExpr::Borrowed(quote! {
                ::core::module_path!()
                    .split(':')
                    .next()
                    .unwrap()
            })),
            _ => None,
        }
    }

    pub fn module_path(&self) -> Option<StringExpr> {
        if let Some(path) = self.get_path() {
            let path_string = path
                .segments
                .pairs()
                .take(path.segments.len() - 1)
                .map(|pair| pair.value().ident.to_string())
                .reduce(|path, ident| path + "::" + &ident)
                .unwrap();

            let path_lit = LitStr::new(&path_string, path.span());
            return Some(StringExpr::from_lit(&path_lit));
        }

        match self {
            Self::Local { .. } => Some(StringExpr::Const(quote! {
                ::core::module_path!()
            })),
            _ => None,
        }
    }

    pub fn type_ident(&self) -> Option<StringExpr> {
        self.get_ident().map(StringExpr::from)
    }

    /// Combines type generics and const generics into one [`StringExpr`].
    ///
    /// This string can be used with a `GenericTypePathCell` in a `TypePath` implementation.
    ///
    /// The `ty_generic_fn` param maps [`TypeParam`]s to [`StringExpr`]s.
    fn reduce_generics(
        generics: &Generics,
        mut ty_generic_fn: impl FnMut(&TypeParam) -> StringExpr,
        vct_reflect_path: &Path,
    ) -> StringExpr {
        let alloc_utils_path = crate::path::alloc_utils_(vct_reflect_path);

        let mut params = generics.params.iter().filter_map(|param| match param {
            GenericParam::Type(type_param) => Some(ty_generic_fn(type_param)),
            GenericParam::Const(const_param) => {
                let ident = &const_param.ident;
                let ty = &const_param.ty;

                Some(StringExpr::Owned(quote! {
                    <#ty as #alloc_utils_path::ToString>::to_string(&#ident)
                }))
            }
            GenericParam::Lifetime(_) => None,
        });

        StringExpr::from_iter(
            params.next().into_iter()
                .chain(params.flat_map(|x| [StringExpr::from_str(", "), x])), 
            vct_reflect_path
        )
    }

    pub fn type_name(&self, vct_reflect_path: &Path) -> StringExpr {
        let type_path_ = crate::path::type_path_(vct_reflect_path);
        match self {
            ReflectTypePath::Primitive(ident) => StringExpr::from(ident),
            Self::Local{ generics, .. } | Self::Foreign{ generics, .. } => {
                let ident = self.type_ident().unwrap();

                if self.impl_with_generic() {
                    let generics = ReflectTypePath::reduce_generics(
                        generics,
                        |TypeParam { ident, .. }| {
                            StringExpr::Borrowed(quote! {
                                <#ident as #type_path_>::type_name()
                            })
                        },
                        vct_reflect_path,
                    );

                    StringExpr::from_iter([
                        ident,
                        StringExpr::from_str("<"),
                        generics,
                        StringExpr::from_str(">"),
                    ], vct_reflect_path)
                } else {
                    ident
                }
            }
        }
    }

    /// Returns a [`StringExpr`] representing the "type path" of the type.
    ///
    /// For `Option<PhantomData>`, this is `"std::option::Option<std::marker::PhantomData>"`.
    pub fn type_path(&self, vct_reflect_path: &Path) -> StringExpr {
        let type_path = crate::path::type_path_(vct_reflect_path);

        match self {
            Self::Primitive(ident) => StringExpr::from(ident),
            Self::Local{ generics, .. } | Self::Foreign{ generics, .. } => {
                let ident = self.type_ident().unwrap();
                let module_path = self.module_path().unwrap();

                if self.impl_with_generic() {
                    let generics = ReflectTypePath::reduce_generics(
                        generics,
                        |TypeParam { ident, .. }| {
                            StringExpr::Borrowed(quote! {
                                <#ident as #type_path>::type_path()
                            })
                        },
                        vct_reflect_path,
                    );

                    StringExpr::from_iter([
                        module_path,
                        StringExpr::from_str("::"),
                        ident,
                        StringExpr::from_str("<"),
                        generics,
                        StringExpr::from_str(">"),
                    ], vct_reflect_path)
                } else {
                    StringExpr::from_iter([module_path, StringExpr::from_str("::"), ident], vct_reflect_path)
                }
            }
        }
    }

}

impl ToTokens for ReflectTypePath<'_> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Local { ident, .. } | Self::Primitive(ident) => ident.to_tokens(tokens),
            Self::Foreign { path, .. } => path.to_tokens(tokens),
        }
    }
}
