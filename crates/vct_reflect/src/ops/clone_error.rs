use alloc::{borrow::Cow, format};
use core::fmt;

use crate::info::FieldId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReflectCloneError {
    /// The type does not have a custom implementation for [`crate::PartialReflect::reflect_clone`].
    NotImplemented { type_path: Cow<'static, str> },
    /// The field cannot be cloned via [`crate::PartialReflect::reflect_clone`].
    FieldNotCloneable {
        type_path: Cow<'static, str>,
        field: FieldId,
        variant: Option<Cow<'static, str>>,
    },
}

impl fmt::Display for ReflectCloneError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { type_path } => {
                write!(f, "`reflect_clone` not implemented for `{type_path}`")
            },
            Self::FieldNotCloneable {
                type_path,
                field,
                variant,
            } => {
                write!(
                    f,
                    "field `{}` cannot be made cloneable for `reflect_clone`",
                    match variant.as_deref() {
                        Some(variant) => format!("{type_path}::{variant}::{field}"),
                        None => format!("{type_path}::{field}"),
                    }
                )
            },
        }
    }
}

impl core::error::Error for ReflectCloneError {}
