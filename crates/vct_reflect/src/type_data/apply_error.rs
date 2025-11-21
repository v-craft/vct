use core::{fmt, error};
use alloc::boxed::Box;
use crate::{
    type_info::{
        ReflectKindError,
        ReflectKind,
    },
};

#[derive(Debug)]
pub enum ApplyError {
    MismatchedKinds {
        from_kind: ReflectKind,
        to_kind: ReflectKind,
    },
    MissingEnumField {
        variant_name: Box<str>,
        field_name: Box<str>,
    },
    MismatchedTypes {
        from_type: Box<str>,
        to_type: Box<str>,
    },
    DifferentSize {
        from_size: usize,
        to_size: usize,
    },
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
            },
            Self::MissingEnumField { variant_name, field_name } => {
                write!(f, "enum variant `{variant_name}` doesn't have a field named `{field_name}`")
            },
            Self::MismatchedTypes { from_type, to_type } => {
                write!(f, "`{from_type}` is not `{to_type}`")
            },
            Self::DifferentSize { from_size, to_size } => {
                write!(f, "attempted to apply type with {from_size} size to a type with {to_size} size")
            },
            Self::UnknownVariant { enum_name, variant_name } => {
                write!(f, "variant with name `{variant_name}` does not exist on enum `{enum_name}`")
            }
        }
    }
}

impl error::Error for ApplyError {}

impl From<ReflectKindError> for ApplyError {
    fn from(value: ReflectKindError) -> Self {
        Self::MismatchedKinds {
            from_kind: value.received,
            to_kind: value.expected,
        }
    }
}
