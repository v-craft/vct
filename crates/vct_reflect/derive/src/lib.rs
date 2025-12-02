use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

pub(crate) static REFLECT_ATTRIBUTE_NAME: &str = "reflect";

mod path;

mod utils;
mod derive_data;
mod impls;


/// Generate all reflection related traits' impl
/// 
/// `#[derive(Reflect)]` will impl following trait:
/// 
/// - impl `TypePath` (`DynamicTypePath` will be auto impl.)
/// - impl `Typed` (`DynamicTyped` and `MaybeTyped` will be auto impl)
/// - impl `Reflect`
/// - impl `GetTypeTraits`
/// - impl `FromReflect`
/// - impl `Struct` for `struct T{ ... }`
/// - impl `TupleStruct` for `struct T(...);`
/// - impl `Enum` for `enum T{ ... }`
/// 
/// `struct T;` will be considered as `Opaque`, and will not impl `Struct`.
/// 
/// ## Trait Impl Switch
/// 
/// If you don't want this macro to impl a certain trait, can add an additional macro: `#[reflect(TraitName = false)]`.
/// If auto-impl is turned off, you need to provide a manual impl of the trait.
/// 
/// For example:
/// 
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(TypePath = false)]
/// struct A { /* ... */ }
/// 
/// impl TypePath for A {
///     /* ... Custom Impl ... */
/// }
/// ```
/// 
/// Actually, `#[reflect(TraitName = true)]` is allowed, but this is meaningless because it is the default value.
/// 
/// Specifically, using `#[reflect(opaque)]` forces the type to be treated as `Opaque`.
/// 
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(opaque)]
/// struct A { /* ..Opaque.. */ }
/// ```
/// 
/// Then, `Struct`/`Tuple`/`Enum`/`TupleStruct` will not be impl. 
/// But note that for Opaque type, `reflect_clone` and other `reflect_xxx` return `None` by default.
/// 
/// ## Trait Method Impl Control
/// 
/// Some impls can also be controlled using macros:
/// 
/// - `#[reflect(type_path = "...path...")]` : Control the impl of `TypePath::type_path`
/// - `#[reflect(type_name = "...name...")]` : Control the impl of `TypePath::type_name`
/// - `#[reflect(clone = Clone/Internal)]` : Control the impl of `PartialReflect::reflect_clone`
///     - By default, `reflect_clone` will return `ReflectCloneError::NotImplemented` for every type.
///     - If it's `Clone`, the type is required to impl `Clone` trait, then will call it directly(usually more efficient).
///     - If it's `Internal`, `reflect_clone` will return `NotCloneable` for `Opaque`,
///       or call internal fields' `reflect_clone` for other type(return `FieldNotCloneable` if some fields are coneable).
/// - `#[reflect(partial_eq = PartialEq/Internal)]` : Control the impl of `PartialReflect::reflect_partial_eq`
///     - By default, `reflect_partial_eq` will return `None` for every type.
///     - If it's `PartialEq`, the type is required to impl `PartialEq` trait, then will call it directly(usually more efficient).
///     - If it's `Internal`, `reflect_partial_eq` will return `None` for `Opaque`,
///       or call internal fields' `reflect_partial_eq` for other type(return `None` if some fields return `None`).
/// - `#[reflect(hash = Hash/Internal)]` : Control the impl of `PartialReflect::reflect_hash`
///     - `reflect_hash` depends on `vct_reflect::reflect_hasher()`, it's a fixed hasher, see `vct_utils::hash::FixedHash`.
///     - By default, `reflect_hash` will return `None` for every type.
///     - If it's `Hash`, the type is required to impl `Hash` trait, then will call it to get u64 directly(usually more efficient).
///     - If it's `Internal`, `reflect_hash` will return `None` for `Opaque`,
///       or call internal fields' `reflect_hash` for other type(return `None` if some fields return `None`).
/// - `#[reflect(debug = Debug)]` : Control the impl of `PartialReflect::reflect_debug`
///     - By default, `reflect_debug` will write `Reflect(#type_path)` for `Opaque`, or `reflect_debug` internal fields for other.
///     - If it's `Debug`, the type is required to impl `Debug` trait, then will call it directly(usually more efficient).
///     - `reflect_debug` does not have `Internal` impl because it is default.
/// 
/// For example:
/// 
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(type_name = "B")]
/// struct A { /* ... */ }
/// ```
/// 
/// If the auto-impl of the certain trait is turned off, the corresponding control options will not work and there will be no negative effects.
/// For exmple, `#[reflect(type_path = "x")]` will not work if `#[reflect(TypePath = false)]`, but it won't cause compile-errors either.
/// 
/// Using custom functions to impl `reflect_xxx` (like `#[reflect(debug = my_debug_func)]`) is interesting, 
/// but it doesn't seem that important and is not currently provided.
/// 
/// ## Custom Attributes
/// 
/// We also support providing custom attributes for types or fields, use `#[reflect(@expr)]`.
/// 
/// For Example:
/// 
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(@Foo::new())]
/// #[reflect(@{6.5 + 1.3 / 2.0})]
/// struct A {
///     #[reflect(@"123")]
///     field: String,
/// }
/// ```
/// 
/// The following of `@` must be an expression.
/// 
/// You need to read these attributes through the `custom_attribles` function of the `TypeInfo` or `XxxInfo`.
/// 
/// Internal storage is type based, and if there are attributes of the same type in a type(or field), 
/// only one will be retained (it is uncertain which one will be retained, most likely declared later).
/// 
/// ### Field Control
/// 
/// Here are some reflection controls for fields only:
/// 
/// - `#[reflect(ignore)]` : Fields will not be reflected.
///     - Using this flag requires that the field type must implement `Default`.
///     - If enabled, the default impl of all reflection methods does not include this field:
///         - Cannot access this field using the `vct_reflect::access::Accessor`.
///         - Will not appear during reflection serialization, and will be replaced with `Default::default` during deserialization.
///         - It will not be used in the `Internal` impl of `reflect_xxx`, But it does not affect the impl based on Trait(eg: `#[reflect(hash = Hash)]`).
/// - `#[reflect(skip_serde)]` : 
///     - Skip this field only during reflected serialization and deserialization.
///     - The fields using this macro must impl `Default` trait (for deserialization).
///     - Not compatible with `#[reflect(ignore)]`, because it's completely ignored during reflection.
/// 
/// For Example:
/// 
/// ```ignore
/// #[derive(Reflect)]
/// struct A {
///     #[reflect(ignore)]
///     _marker: PhantomData,
///     #[reflect(skip_serde)]
///     note: &'static str,
///     content: u64,
/// }
/// ```
/// 
/// ### Alias Control
/// 
/// Rust requires that when implementing a trait, either the type or the trait must be defined in the current crate.
/// Alias reflection is designed to address this issue by treating your specified type as an alias of another type during reflection,
/// providing conversion support between the two.
/// 
/// You must strictly ensure that the two types of memory layouts are completely consistent!!
/// 
/// For Example:
/// 
/// ```ignore
/// struct B {
///     content: u64,
/// }
/// 
/// #[derive(Reflect)]
/// #[reflect(alias = B)]
/// #[repr(transparent)] // ensures the same memory layout
/// struct A(B);
/// ```
/// 
/// 
/// ### Docs Control
/// 
/// Should enable `reflect_docs` feature.
/// 
/// By default, `reflect_docs` collects the content of all `#[doc = "..."]` macros and concat them into a document.
/// (This includes document comments `/// ...`.)
/// 
/// But if you can explicitly use `#[reflect(docs = "")]`, then it won't use document comments as the docs of reflect info.
/// 
/// This macro has no negative effects when the `reflect_deocs` feature is not enabled.
/// 
/// For Example:
/// 
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(docs = "This is a struct.")]
/// struct A {
///     #[reflect(skip_serde)]
///     #[reflect(docs = "This is a field.")]
///     #[reflect(docs = "This field should not be serialized.")]
///     note: &'static str,
///     content: u64,
/// }
/// ```
/// 
/// Note: use `#[reflect(docs = "")]` to close the docs of a certain type.
/// 
/// ### Auto Register
/// 
/// Should enable `auto_register` feature.
/// 
/// By default, types are not automatically registered (even if this `auto_register` is enabled).
/// 
/// You need to explicitly add macro `#[reflect(auto_register)]` on the type.
/// 
/// This macro has no negative effects when the `auto_register` feature is not enabled.
/// 
/// For Example:
/// 
/// ```ignore
/// #[derive(Reflect)]
/// #[reflect(auto_register)]
/// struct A {
///     /* ... */
/// }
/// ```
#[proc_macro_derive(Reflect, attributes(reflect))]
pub fn derive_full_reflect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impls::match_reflect_impls(ast, ImplSourceKind::DeriveLocalType)
}

/// A replacement for `#[derive(Reflect)]` to be used with foreign types which
/// the definitions of cannot be altered. The usage is similar.
/// 
/// For example:
/// 
/// ```ignore
/// impl_full_reflect!{
///     #[reflect(@{6.5 + 1.3 / 2.0})]
///     struct A {
///         field: String,
///     }
/// };
/// ```
/// 
/// Usually used to implement reflection for standard library types:
/// 
/// ```ignore
/// impl_full_reflect!{
///     #[reflect(type_name = "u8")]
///     #[reflect(type_path = "u8")]
///     #[reflect(Opaque = true)]
///     #[reflect(hash = Hash)]
///     #[reflect(clone = Clone)]
///     #[reflect(debug = Debug)]
///     #[reflect(partial_eq = PartialEq)]
///     struct u8;
/// };
/// ```
#[proc_macro]
pub fn impl_full_reflect(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    impls::match_reflect_impls(ast, ImplSourceKind::ImplForeignType)
}

/// How the macro was invoked.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ImplSourceKind {
    /// Using `impl_reflect!`.
    ImplForeignType,
    /// Using `#[derive(...)]`.
    DeriveLocalType,
}
