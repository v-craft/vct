use crate::info::{TypeInfo, TypePath};

/// A static accessor to compile-time type information.
pub trait Typed: TypePath {
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
