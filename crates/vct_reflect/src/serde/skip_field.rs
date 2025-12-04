use alloc::boxed::Box;

use crate::{
    Reflect,
    cell::NonGenericTypeInfoCell,
    info::{DynamicTypePath, OpaqueInfo, ReflectKind, TypeInfo, TypePath, Typed},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
    reflect::impl_cast_reflect_fn,
};

/// A custom attribute use to skip serialization and deserialization of fields.
pub struct SkipSerde(pub(crate) Option<Box<dyn Reflect>>);

impl SkipSerde {
    #[inline]
    pub fn default() -> Self {
        Self(None)
    }

    pub fn with<T: Reflect>(val: T) -> Self {
        Self(Some(Box::new(val).into_reflect()))
    }
}

impl TypePath for SkipSerde {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::serde::SkipSerde"
    }

    #[inline]
    fn type_name() -> &'static str {
        "SkipSerde"
    }

    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("SkipSerde")
    }

    #[inline]
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }

    #[inline]
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::serde")
    }
}

impl Typed for SkipSerde {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl Reflect for SkipSerde {
    impl_cast_reflect_fn!();

    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Opaque
    }

    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Opaque(self)
    }

    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Opaque(self)
    }

    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Opaque(self)
    }

    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        // This function should not be used.
        Err(ApplyError::MismatchedTypes {
            from_type: value.reflect_type_path().into(),
            to_type: self.reflect_type_path().into(),
        })
    }

    fn to_dynamic(&self) -> Box<dyn Reflect> {
        match &self.0 {
            Some(val) => Box::new(SkipSerde(Some(val.to_dynamic()))),
            None => Box::new(SkipSerde(None)),
        }
    }

    fn reflect_debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.0 {
            Some(_) => f.write_str("SkipSerde::clone(?)"),
            None => f.write_str("SkipSerde::default"),
        }
    }
}
