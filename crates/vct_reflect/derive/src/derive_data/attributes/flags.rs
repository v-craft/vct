use proc_macro2::Span;
use syn::Path;

/// A struct used to control whether a trait needs to be implemented.
#[derive(Clone)]
pub(crate) struct TraitImplFlags {
    /// Default is `true`, use `#[reflect(TypePath = false)]`  to disable it.
    /// Then Users can(must) impl it in a more customized way.
    pub(crate) impl_type_path: bool,
    /// Default is `true`, use `#[reflect(Typed = false)]`  to disable it.
    /// Then Users can(must) impl it in a more customized way.
    pub(crate) impl_typed: bool,
    /// Default is `true`, use `#[reflect(Reflect = false)]`  to disable it.
    /// Then Users can(must) impl it in a more customized way.
    pub(crate) impl_reflect: bool,
    /// Default is `true`, use `#[reflect(GetTypeTraits = false)]`  to disable it.
    /// Then Users can(must) impl it in a more customized way.
    pub(crate) impl_get_type_traits: bool,
    /// Default is `true`, use `#[reflect(FromReflect = false)]`  to disable it.
    /// Then Users can(must) impl it in a more customized way.
    pub(crate) impl_from_reflect: bool,
    /// Default is `true`, use `#[reflect(Struct = false)]`  to disable it.
    /// Even if it is true, it only takes effect when the type is correct.
    pub(crate) impl_struct: bool,
    /// Default is `true`, use `#[reflect(TupleStruct = false)]`  to disable it.
    /// Even if it is true, it only takes effect when the type is correct.
    pub(crate) impl_tuple_struct: bool,
    /// Default is `true`, use `#[reflect(Tuple = false)]`  to disable it.
    /// Even if it is true, it only takes effect when the type is correct.
    pub(crate) impl_tuple: bool,
    /// Default is `true`, use `#[reflect(Enum = false)]`  to disable it.
    /// Even if it is true, it only takes effect when the type is correct.
    pub(crate) impl_enum: bool,
}

impl Default for TraitImplFlags {
    fn default() -> Self {
        Self {
            impl_type_path: true, 
            impl_typed: true, 
            impl_reflect: true, 
            impl_get_type_traits: true, 
            impl_from_reflect: true, 
            impl_struct: true, 
            impl_tuple_struct: true, 
            impl_tuple: true, 
            impl_enum: true 
        }
    }
}

#[derive(Clone, Default)]
pub(crate) enum MethodFlag {
    /// By default, no implementation is provided. (except for `reflect_debug`)
    #[default]
    Default,
    /// This option will be implemented through a subfield (and its own type name). 
    /// When there are sub fields that are not supported, this type is also not supported.  
    /// For `reflect_debug`, this is eq to `Default`.  
    /// For example: `#[reflect(clone = Internal)]`.
    Internal(Span),
    /// This option will implement the function through the same name Trait(the type must implemented this trait), 
    /// This is usually more efficient than `Internal`.  
    /// For example: `#[reflect(clone = Clone)]`.
    Trait(Span),
    /// This option allows users to implement it using the specified function,
    /// but it is important to ensure the correctness of the function parameters.  
    /// For example: `#[reflect(clone = utils::my_clone_fun)]`
    Custom(Path, Span),
}

#[derive(Clone, Default)]
pub(crate) struct MethodImplFlags {
    pub reflect_clone: MethodFlag,
    pub reflect_debug: MethodFlag,
    pub reflect_hash: MethodFlag,
    pub reflect_partial_eq: MethodFlag,
    pub reflect_default: MethodFlag,
}



