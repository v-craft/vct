use alloc::boxed::Box;
use crate::{
    ops::{
        Struct,
    },
    info::{
        ReflectKind, ReflectKindError,
    }
};

pub enum ReflectRef<'a> {
    Struct(&'a dyn Struct),
    Todo,
}


pub enum ReflectMut<'a> {
    Struct(&'a mut dyn Struct),
    Todo,
}

pub enum ReflectOwned {
    Struct(Box<dyn Struct>),
    Todo,
}

macro_rules! impl_kind_fn {
    () => {
        pub fn kind(&self) -> ReflectKind {
            match self {
                Self::Struct(_) => ReflectKind::Struct,
                _ => ReflectKind::Opaque,
                // Self::TupleStruct(_) => ReflectKind::TupleStruct,
                // Self::Tuple(_) => ReflectKind::Tuple,
                // Self::List(_) => ReflectKind::List,
                // Self::Array(_) => ReflectKind::Array,
                // Self::Map(_) => ReflectKind::Map,
                // Self::Set(_) => ReflectKind::Set,
                // Self::Enum(_) => ReflectKind::Enum,
                // Self::Opaque(_) => ReflectKind::Opaque,
            }
        }
    };
}


macro_rules! impl_cast_fn {
    ($name:ident : Opaque => $retval:ty) => {
        pub fn $name(self) -> Result<$retval, ReflectKindError> {
            match self {
                Self::Opaque(value) => Ok(value),
                _ => Err(ReflectKindError {
                    expected: ReflectKind::Opaque,
                    received: self.kind(),
                }),
            }
        }
    };
    ($name:ident : $kind:ident => $retval:ty) => {
        pub fn $name(self) -> Result<$retval, ReflectKindError> {
            match self {
                Self::$kind(value) => Ok(value),
                _ => Err(ReflectKindError {
                    expected: ReflectKind::$kind,
                    received: self.kind(),
                }),
            }
        }
    };
}

impl<'a> ReflectRef<'a> {
    impl_kind_fn!();
    impl_cast_fn!(as_struct: Struct => &'a dyn Struct);
}

impl<'a> ReflectMut<'a> {
    impl_kind_fn!();
    impl_cast_fn!(as_struct: Struct => &'a mut dyn Struct);
}

impl ReflectOwned {
    impl_kind_fn!();
    impl_cast_fn!(into_struct: Struct => Box<dyn Struct>);
}
