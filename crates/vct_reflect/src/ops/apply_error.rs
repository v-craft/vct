use crate::info::{ReflectKind, ReflectKindError};
use alloc::boxed::Box;
use core::{error, fmt};

/// A enumeration of all error outcomes
/// that might happen when running [`try_apply`](crate::PartialReflect::try_apply).
#[derive(Debug)]
pub enum ApplyError {
    /// Attempted to apply the wrong [kind](ReflectKind) to a type, e.g. a struct to an enum.
    MismatchedKinds {
        from_kind: ReflectKind,
        to_kind: ReflectKind,
    },
    /// Enum variant that we tried to apply to was missing a field.
    MissingEnumField {
        variant_name: Box<str>,
        field_name: Box<str>,
    },
    /// Tried to apply incompatible types.
    MismatchedTypes {
        from_type: Box<str>,
        to_type: Box<str>,
    },
    /// Attempted to apply an [array-like] type to another of different size, e.g. a [u8; 4] to [u8; 3].
    DifferentSize { from_size: usize, to_size: usize },
    /// The enum we tried to apply to didn't contain a variant with the give name.
    UnknownVariant {
        enum_name: Box<str>,
        variant_name: Box<str>,
    },
}

impl fmt::Display for ApplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MismatchedKinds { from_kind, to_kind } => {
                write!(f, "attempted to apply `{from_kind}` to `{to_kind}`")
            }
            Self::MissingEnumField {
                variant_name,
                field_name,
            } => {
                write!(
                    f,
                    "enum variant `{variant_name}` doesn't have a field named `{field_name}`"
                )
            }
            Self::MismatchedTypes { from_type, to_type } => {
                write!(f, "`{from_type}` is not `{to_type}`")
            }
            Self::DifferentSize { from_size, to_size } => {
                write!(
                    f,
                    "attempted to apply type with {from_size} size to a type with {to_size} size"
                )
            }
            Self::UnknownVariant {
                enum_name,
                variant_name,
            } => {
                write!(
                    f,
                    "variant with name `{variant_name}` does not exist on enum `{enum_name}`"
                )
            }
        }
    }
}

impl error::Error for ApplyError {}

impl From<ReflectKindError> for ApplyError {
    #[inline]
    fn from(value: ReflectKindError) -> Self {
        Self::MismatchedKinds {
            from_kind: value.received,
            to_kind: value.expected,
        }
    }
}
