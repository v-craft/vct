
use core::{fmt, any::{Any, TypeId}};
use alloc::{
    boxed::Box, format,
    string::{String, ToString}
};

use crate::{
    Reflect, PartialReflect, FromReflect,
    info::{TypePath, Typed, TypeInfo, OpaqueInfo, ReflectKind},
    cell::NonGenericTypeInfoCell,
    ops::{ApplyError, ReflectRef, ReflectMut, ReflectOwned, ReflectCloneError},
    registry::{TypeRegistry, TypeTraits, GetTypeTraits}
};


macro_rules! impl_native_number {
    ($name:ident, $str_name:literal) => {
        impl TypePath for $name {
            #[inline]
            fn type_path() -> &'static str {
                $str_name
            }
            #[inline]
            fn type_name() -> &'static str {
                $str_name
            }
        }

        impl Typed for $name {
            fn type_info() -> &'static TypeInfo {
                static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
                CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<$name>()))
            }
        }

        impl PartialReflect for $name {
            #[inline]
            fn get_target_type_info(&self) -> Option<&'static TypeInfo> {
                Some(<Self as Typed>::type_info())
            }

            #[inline]
            fn as_partial_reflect(&self) -> &dyn PartialReflect {
                self
            }

            #[inline]
            fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
                self
            }

            #[inline]
            fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
                self
            }

            #[inline]
            fn try_as_reflect(&self) -> Option<&dyn Reflect> {
                Some(self)
            }

            #[inline]
            fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
                Some(self)
            }

            #[inline]
            fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
                Ok(self)
            }

            fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
                todo!()
                // let id = value.type_id();
                // if TypeId::of::<$name>() == id {
                //     // TODO: Replace to downcast_ref_uncheck
                //     *self = *<dyn Any>::downcast_ref::<$name>(value).unwrap();
                // } else if TypeId::of::<u8> == id {
                //     *self = (*<dyn Any>::downcast_ref::<u8>(value)).unwrap() as $name;
                // } else if TypeId::of::<i8> == id {

                // }

                // let kind = value.reflect_kind();
                // if kind != ReflectKind::Opaque {
                //     return Err(ApplyError::MismatchedKinds{
                //         from_kind: kind,
                //         to_kind: ReflectKind::Opaque,
                //     });
                // }

                // let from_type: Box<str> = {
                //     match value.get_target_type_info() {
                //         Some(info) => {
                //             info.type_path().into()
                //         },
                //         None => {
                //             format!("UnknownType::{}", kind).into_boxed_str()
                //         }
                //     }
                // };

                // Err(ApplyError::MismatchedTypes {
                //     from_type,
                //     to_type: $str_name.into(),
                // })
            }

            #[inline]
            fn reflect_kind(&self) -> ReflectKind {
                ReflectKind::Opaque
            }

            #[inline]
            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::Opaque(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Opaque(self)
            }

            #[inline]
            fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                ReflectOwned::Opaque(self)
            }

            #[inline]
            fn to_dynamic(&self) -> Box<dyn PartialReflect> {
                // Do not use default impl: faster
                Box::new(*self)
            }

            fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
                match other.try_downcast_ref::<$name>() {
                    Some(val) => {
                        Some(PartialEq::eq(self, val))
                    },
                    None => None,
                }
            }

            #[inline]
            fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
                Ok(Box::new(*self))
            }

            #[inline]
            fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                // Faster and may reduce compilation time.
                write!(f, "{}({})", $str_name, self)
            }
        }

        impl Reflect for $name {
            #[inline]
            fn as_reflect(&self) -> &dyn Reflect {
                self
            }

            #[inline]
            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                self
            }

            #[inline]
            fn into_reflect(self: Box<Self>) -> Box<dyn Reflect> {
                self
            }

            #[inline]
            fn set(&mut self, value: Box<dyn Reflect>) -> Result<(), Box<dyn Reflect>> {
                *self = value.take()?;
                Ok(())
            }
        }

        impl GetTypeTraits for $name {
            #[inline]
            fn get_type_traits() -> TypeTraits {
                TypeTraits::of::<$name>()
            }

            #[inline]
            fn register_dependencies(_registry: &mut TypeRegistry) {}
        }

        impl FromReflect for $name {
            fn from_reflect(other: &dyn PartialReflect) -> Option<Self> {
                todo!()
            }
        }
    };
}



impl_native_number!(u8, "u8");




