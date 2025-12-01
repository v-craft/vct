/// How the macro was invoked.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ImplSourceKind {
    /// Using `impl_reflect!`.
    ImplAliasType,
    /// Using `#[derive(...)]`.
    DeriveLocalType,
    /// Using `#[reflect_alias]`.
    AliasReflect,
}

/// Which trait the macro explicitly implements.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) enum ImplTraitKind {
    Reflect,
    FromReflect,
    TypePath,
}

/// The provenance of a macro invocation.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(crate) struct ReflectProvenance {
    pub source: ImplSourceKind,
    pub trait_: ImplTraitKind,
}

