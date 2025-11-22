use core::fmt;
use alloc::{borrow::Cow, string::String, format};
use crate::info::FieldId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReflectCloneError {
    NotImplemented {
        type_path: Cow<'static, str>,
    },
    NotCloneable {
        type_path: Cow<'static, str>,
    },
    FieldNotCloneable {
        field: FieldId,
        variant: Option<Cow<'static, str>>,
        container_type_path: Cow<'static, str>,
    },
    FailedDowncast {
        expected: Cow<'static, str>,
        received: Cow<'static, str>,
    },
}

fn full_path(
    field: &FieldId,
    variant: Option<&str>,
    container_type_path: &str,
) -> String {
    match variant {
        Some(variant) => format!("{container_type_path}::{variant}::{field}"),
        None => format!("{container_type_path}::{field}"),
    }
}

impl fmt::Display for ReflectCloneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { type_path } => {
                write!(f, "`PartialReflect::reflect_clone` not implemented for `{type_path}`")
            },
            Self::NotCloneable { type_path } => {
                write!(f, "`{type_path}` cannot be made cloneable for `PartialReflect::reflect_clone`")
            },
            Self::FieldNotCloneable { field, variant, container_type_path } => {
                write!(
                    f,
                    "field `{}` cannot be made cloneable for `PartialReflect::reflect_clone` (are you missing a `#[reflect(clone)]` attribute?)",
                    full_path(field, variant.as_deref(), container_type_path)
                )
            },
            Self::FailedDowncast { expected, received } => {
                write!(f, "expected downcast to `{expected}`, but received `{received}`")
            },
        }
    }
}

impl core::error::Error for ReflectCloneError {}
