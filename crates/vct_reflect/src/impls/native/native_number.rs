
use core::{fmt, any::TypeId};
use alloc::boxed::Box;

use crate::{
    Reflect, FromReflect,
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

            #[inline]
            fn represented_type_info(&self) -> Option<&'static TypeInfo> {
                Some(<Self as Typed>::type_info())
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
            fn to_dynamic(&self) -> Box<dyn Reflect> {
                // Do not use default impl: faster
                Box::new(*self)
            }

            fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
                let id = value.type_id();
                if TypeId::of::<$name>() == id {
                    // TODO: Replace to downcast_ref_uncheck
                    *self = *value.downcast_ref::<$name>().unwrap();
                } else if TypeId::of::<u8>() == id {
                    *self = *value.downcast_ref::<u8>().unwrap() as $name;
                } else if TypeId::of::<i8>() == id {
                    *self = *value.downcast_ref::<i8>().unwrap() as $name;
                } else if TypeId::of::<u16>() == id {
                    *self = *value.downcast_ref::<u16>().unwrap() as $name;
                } else if TypeId::of::<i16>() == id {
                    *self = *value.downcast_ref::<i16>().unwrap() as $name;
                } else if TypeId::of::<u32>() == id {
                    *self = *value.downcast_ref::<u32>().unwrap() as $name;
                } else if TypeId::of::<i32>() == id {
                    *self = *value.downcast_ref::<i32>().unwrap() as $name;
                } else if TypeId::of::<u64>() == id {
                    *self = *value.downcast_ref::<u64>().unwrap() as $name;
                } else if TypeId::of::<i64>() == id {
                    *self = *value.downcast_ref::<i64>().unwrap() as $name;
                } else if TypeId::of::<u128>() == id {
                    *self = *value.downcast_ref::<u128>().unwrap() as $name;
                } else if TypeId::of::<i128>() == id {
                    *self = *value.downcast_ref::<i128>().unwrap() as $name;
                } else if TypeId::of::<f32>() == id {
                    *self = *value.downcast_ref::<f32>().unwrap() as $name;
                } else if TypeId::of::<f64>() == id {
                    *self = *value.downcast_ref::<f64>().unwrap() as $name;
                } else if TypeId::of::<isize>() == id {
                    *self = *value.downcast_ref::<isize>().unwrap() as $name;
                } else if TypeId::of::<usize>() == id {
                    *self = *value.downcast_ref::<usize>().unwrap() as $name;
                }

                let kind = value.reflect_kind();
                if kind != ReflectKind::Opaque {
                    return Err(ApplyError::MismatchedKinds{
                        from_kind: kind,
                        to_kind: ReflectKind::Opaque,
                    });
                }

                Err(ApplyError::MismatchedTypes {
                    from_type: value.reflect_type_info().type_path().into(),
                    to_type: $str_name.into(),
                })
            }

            fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
                match other.downcast_ref::<$name>() {
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

        impl GetTypeTraits for $name {
            #[inline]
            fn get_type_traits() -> TypeTraits {
                TypeTraits::of::<$name>()
            }

            #[inline]
            fn register_dependencies(_registry: &mut TypeRegistry) {}
        }

        impl FromReflect for $name {
            fn from_reflect(value: &dyn Reflect) -> Option<Self> {
                let id = value.type_id();
                if TypeId::of::<$name>() == id {
                    // TODO: Replace to downcast_ref_uncheck
                    Some(*value.downcast_ref::<$name>().unwrap())
                } else if TypeId::of::<u8>() == id {
                    Some(*value.downcast_ref::<u8>().unwrap() as Self)
                } else if TypeId::of::<i8>() == id {
                    Some(*value.downcast_ref::<i8>().unwrap() as Self)
                } else if TypeId::of::<u16>() == id {
                    Some(*value.downcast_ref::<u16>().unwrap() as Self)
                } else if TypeId::of::<i16>() == id {
                    Some(*value.downcast_ref::<i16>().unwrap() as Self)
                } else if TypeId::of::<u32>() == id {
                    Some(*value.downcast_ref::<u32>().unwrap() as Self)
                } else if TypeId::of::<i32>() == id {
                    Some(*value.downcast_ref::<i32>().unwrap() as Self)
                } else if TypeId::of::<u64>() == id {
                    Some(*value.downcast_ref::<u64>().unwrap() as Self)
                } else if TypeId::of::<i64>() == id {
                    Some(*value.downcast_ref::<i64>().unwrap() as Self)
                } else if TypeId::of::<u128>() == id {
                    Some(*value.downcast_ref::<u128>().unwrap() as Self)
                } else if TypeId::of::<i128>() == id {
                    Some(*value.downcast_ref::<i128>().unwrap() as Self)
                } else if TypeId::of::<f32>() == id {
                    Some(*value.downcast_ref::<f32>().unwrap() as Self)
                } else if TypeId::of::<f64>() == id {
                    Some(*value.downcast_ref::<f64>().unwrap() as Self)
                } else if TypeId::of::<isize>() == id {
                    Some(*value.downcast_ref::<isize>().unwrap() as Self)
                } else if TypeId::of::<usize>() == id {
                    Some(*value.downcast_ref::<usize>().unwrap() as Self)
                } else {
                    None
                }
            }
        }
    };
}



impl_native_number!(u8, "u8");
impl_native_number!(i8, "i8");
impl_native_number!(u16, "u16");
impl_native_number!(i16, "i16");
impl_native_number!(u32, "u32");
impl_native_number!(i32, "i32");
impl_native_number!(u64, "u64");
impl_native_number!(i64, "i64");
impl_native_number!(u128, "u128");
impl_native_number!(i128, "i128");
impl_native_number!(f32, "f32");
impl_native_number!(f64, "f64");
impl_native_number!(usize, "usize");
impl_native_number!(isize, "isize");
