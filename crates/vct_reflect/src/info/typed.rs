use crate::{
    PartialReflect, Reflect,
    info::{TypeInfo, TypePath},
};

/// A static accessor to compile-time type information.
pub trait Typed: Reflect + TypePath {
    fn type_info() -> &'static TypeInfo;
}

/// Dynamic dispatch for [`Typed`].
pub trait DynamicTyped {
    /// See [`Typed::type_info`].
    fn reflect_type_info(&self) -> &'static TypeInfo;
}

impl<T: Typed> DynamicTyped for T {
    #[inline]
    fn reflect_type_info(&self) -> &'static TypeInfo {
        Self::type_info()
    }
}

/// A wrapper trait around [`Typed`].
pub trait MaybeTyped: PartialReflect {
    #[inline]
    fn maybe_type_info() -> Option<&'static TypeInfo> {
        None
    }
}

impl<T: Typed> MaybeTyped for T {
    #[inline]
    fn maybe_type_info() -> Option<&'static TypeInfo> {
        Some(T::type_info())
    }
}

// ↓ At the definition of the type itself
// impl MaybeTyped for DynamicEnum {}
// impl MaybeTyped for DynamicSet {}
// impl MaybeTyped for DynamicTupleStruct {}
// impl MaybeTyped for DynamicStruct {}
// impl MaybeTyped for DynamicMap {}
// impl MaybeTyped for DynamicList {}
// impl MaybeTyped for DynamicArray {}
// impl MaybeTyped for DynamicTuple {}
