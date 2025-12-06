#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

use crate::derive_data::{ReflectMeta, ReflectOpaqueParser, TypeAttributes, TypePathParser};

pub(crate) static REFLECT_ATTRIBUTE_NAME: &str = "reflect";

mod path;

mod utils;
mod derive_data;
mod impls;


/// # Example
/// 
/// ```rust, ignore
/// #[derive(Reflect)]
/// struct Foo { /* ... */ }
/// ```
/// 
/// `#[derive(Reflect)]` implements:
/// 
/// - `TypePath`
/// - `Typed`
/// - `Reflect`
/// - `GetTypeTraits`
/// - `FromReflect`
/// - `Struct` for `struct T { ... }`
/// - `TupleStruct` for `struct T(...);`
/// - `Enum` for `enum T { ... }`
/// 
/// Special case: `struct A;` is treated as `Opaque` and will not implement `Struct`.
/// 
/// # Implementation control
/// 
/// You can disable specific impls via attributes; then you must provide them manually.
/// 
/// ```rust, ignore
/// #[derive(Reflect)]
/// #[reflect(TypePath = false, Typed = false)]
/// struct Foo { /* ... */ }
/// ```
/// 
/// All of the above toggles can be turned off; turning them on explicitly is meaningless because it is the default.
/// 
/// Because `TypePath` often needs customization, an attribute is provided to override the path:
/// 
/// ```rust, ignore
/// #[derive(Reflect)]
/// #[reflect(type_path = "you::me::Foo")]
/// struct Foo { /* ... */ }
/// ```
/// 
/// `Opaque` is a special attribute that forces the type to be treated as `Opaque` instead of `Struct`, etc.
/// 
/// ```rust, ignore
/// #[derive(Reflect)]
/// #[reflect(Opaque)] // error
/// struct Foo { /* ... */ }
/// ```
/// 
/// Unit structs like `struct A;` are also treated as `Opaque`. They have no internal data, so the macro can auto-generate `reflect_clone`, etc. But if you mark a type as `Opaque` yourself, the macro will not inspect its fields; therefore `Opaque` types are required to at least implement `Clone`.
/// 
/// ```rust, ignore
/// #[derive(Reflect, Clone)]
/// #[reflect(Opaque, clone)] // error
/// struct Foo { /* ... */ }
/// ```
/// 
/// # Using standard traits
/// 
/// If the type implements traits like `Hash` or `Clone`, the reflection impls can be simplified (often much faster). The macro cannot know this, so it does not assume them by default. Use attributes to declare availability so the macro can optimize. As noted, `Opaque` types must support `Clone`, so implement it and mark with `clone`.
/// 
/// ```rust, ignore
/// #[derive(Reflect)]
/// #[reflect(Opaque, clone, hash)] // error
/// struct Foo { /* ... */ }
/// // impl Clone, Hash ...
/// ```
/// 
/// Available flags:
/// 
/// - `clone`: std::Clone
/// - `default`: std::Default
/// - `hash`: std::Hash
/// - `partial_eq`: std::PartialEq
/// - `serialize`: serde::Serialize
/// - `deserialize`: serde::Deserialize
/// 
/// `auto_register` is special: with the feature enabled, marked types auto-register.
/// 
/// Two convenience bundles enable multiple flags at once:
/// 
/// - `serde`: `serialize` + `deserialize` + `auto_register`
/// - `full`: all seven above (including `auto_register`)
/// 
/// # Docs reflection
/// 
/// Enable the `reflect_docs` feature to include docs in type info. By default the macro collects `#[doc = "..."]` (including `///` comments).
/// 
/// Use `#[reflect(docs = false)]` to disable doc collection for a type.
/// 
/// Use `#[reflect(docs = "...")]` to override with custom docs; when present, the macro ignores `#[doc = "..."]`.
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_full_reflect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impls::match_reflect_impls(ast, ImplSourceKind::DeriveLocalType)
}

/// Implements reflection for foreign types, requiring full type info and field access. 
/// Due to the orphan rule, this is typically used inside the reflection crate itself.
/// 
/// ```rust, ignore
/// impl_reflect! {
///     #[reflect(full)]
///     enum Option<T> {
///         Some(T),
///         None,
///     }
/// }
/// ```
/// 
/// See: [`derive Reflect`](derive_full_reflect)
#[proc_macro]
pub fn impl_reflect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impls::match_reflect_impls(ast, ImplSourceKind::ImplForeignType)
}

/// How the macro was invoked.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ImplSourceKind {
    /// Using `impl_full_reflect!`.
    ImplForeignType,
    /// Using `#[derive(...)]`.
    DeriveLocalType,
}

/// Simplified macro for `Opaque` types. Syntax: `(in module_path as alias_name) ident (..attrs..)`.
/// 
/// Examples:
/// 
/// ```rust, ignore
/// impl_reflect_opaque!(u64 (full));
/// impl_reflect_opaque!(String (clone, debug, TypePath = false, docs = "hello"));
/// impl_reflect_opaque!((in utils::time) Instant (clone));
/// impl_reflect_opaque!((in utils::time as Ins) Instant (clone));
/// ```
/// 
/// This macro always implies `Opaque`, so `clone` is required.
/// 
/// See: [`derive Reflect`](derive_full_reflect)
#[proc_macro]
pub fn impl_reflect_opaque(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input with ReflectOpaqueParser::parse);

    // let default_name = &def.type_path.segments.last().unwrap().ident;
    let parser = {
        if op.type_path.leading_colon.is_none() && op.custom_path.is_none() {
            TypePathParser::Primitive(&op.type_ident)
        } else {
            TypePathParser::Foreign {
                path: &op.type_path,
                custom_path: op.custom_path,
                generics: &op.generics,
            }
        }
    };

    let assert_tokens = match &parser {
        TypePathParser::Primitive(_) => {
            let ident = &op.type_ident;
            quote! {
                mod __assert_primitive_ident {
                    type AssertIdentValidity = #ident;
                }
            }
        },
        _ => utils::empty(),
    };

    let meta = ReflectMeta::new(op.attrs, parser);

    let reflect_impls = impls::impl_opaque(&meta);

    quote! {
        const _: () = {
            #assert_tokens

            #reflect_impls
        };
    }.into()
}
