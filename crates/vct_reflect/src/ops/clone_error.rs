use alloc::{borrow::Cow, format};
use core::fmt;

use crate::info::FieldId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReflectCloneError {
    /// The type does not have a custom implementation for [`crate::PartialReflect::reflect_clone`].
    NotImplemented { type_path: Cow<'static, str> },
    /// The type cannot be cloned via [`crate::PartialReflect::reflect_clone`].
    NotCloneable { type_path: Cow<'static, str> },
    /// The field cannot be cloned via [`crate::PartialReflect::reflect_clone`].
    FieldNotCloneable {
        field: FieldId,
        variant: Option<Cow<'static, str>>,
        container_type_path: Cow<'static, str>,
    },
    /// Could not downcast to the expected type.
    FailedDowncast {
        expected: Cow<'static, str>,
        received: Cow<'static, str>,
    },
}

impl fmt::Display for ReflectCloneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { type_path } => {
                write!(
                    f,
                    "`PartialReflect::reflect_clone` not implemented for `{type_path}`"
                )
            }
            Self::NotCloneable { type_path } => {
                write!(
                    f,
                    "`{type_path}` cannot be made cloneable for `PartialReflect::reflect_clone`"
                )
            }
            Self::FieldNotCloneable {
                field,
                variant,
                container_type_path,
            } => {
                write!(
                    f,
                    "field `{}` cannot be made cloneable for `PartialReflect::reflect_clone` (are you missing a `#[reflect(clone)]` attribute?)",
                    match variant.as_deref() {
                        Some(variant) => format!("{container_type_path}::{variant}::{field}"),
                        None => format!("{container_type_path}::{field}"),
                    }
                )
            }
            Self::FailedDowncast { expected, received } => {
                write!(
                    f,
                    "expected downcast to `{expected}`, but received `{received}`"
                )
            }
        }
    }
}

impl core::error::Error for ReflectCloneError {}
