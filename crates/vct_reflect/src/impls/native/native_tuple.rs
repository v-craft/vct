//! Implement reflection traits for tuples with a field count of 12 or less.
//!
//! - [`TypePath`] -> [`DynamicTypePath`]
//! - [`Typed`] -> [`DynamicTyped`]
//! - [`Tuple`]
//! - [`Reflect`]
//! - [`GetTypeTraits`]
//! - [`FromReflect`]
//!
//! [`DynamicTypePath`]: crate::info::DynamicTypePath
//! [`DynamicTyped`]: crate::info::DynamicTyped

use core::fmt;
use crate::{
    FromReflect, Reflect,
    cell::{GenericTypeInfoCell, GenericTypePathCell},
    info::{ReflectKind, TupleInfo, TypeInfo, TypePath, Typed, UnnamedField},
    ops::{
        ApplyError, ReflectCloneError, ReflectMut, ReflectOwned, ReflectRef, Tuple, TupleFieldIter,
        tuple_partial_eq, tuple_try_apply, tuple_debug,
    },
    registry::{GetTypeTraits, TypeRegistry, TypeTraits},
    impls::concat,
};
use alloc::{boxed::Box, vec, vec::Vec};
use vct_utils::range_invoke;

macro_rules! impl_type_path_tuple {
    (0: []) => {
        impl TypePath for () {
            #[inline]
            fn type_path() -> &'static str {
                "()"
            }
            #[inline]
            fn type_name() -> &'static str {
                "()"
            }
        }
    };
    (1: [$zero:ident]) => {
        impl<$zero: TypePath> TypePath for ($zero,) {
            fn type_path() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    // TODO: Replace to `alloc::slice::Concat` .
                    concat(&["(" , $zero::type_path() , ",)"])
                })
            }

            fn type_name() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    // TODO: Replace to `alloc::slice::Concat` .
                    concat(&["(" , $zero::type_name() , ",)"])
                })
            }
        }
    };
    ($_:literal: [$zero:ident, $($index:ident),*]) => {
        impl<$zero: TypePath, $($index: TypePath),*> TypePath for ($zero, $($index),*) {
            fn type_path() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    // TODO: Replace to `alloc::slice::Concat` .
                    concat(&["(", $zero::type_path() $(, ", ", $index::type_path())* , ")"])
                })
            }

            fn type_name() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    // TODO: Replace to `alloc::slice::Concat` .
                    concat(&["(", $zero::type_name() $(, ", ", $index::type_name())* , ")"])
                })
            }
        }
    };
}

range_invoke!(impl_type_path_tuple, 12);

macro_rules! impl_reflect_tuple {
    ($num:literal : [$($index:tt : $name:ident),*]) => {
        impl<$($name: Reflect + Typed),*> Typed for ($($name,)*) {
            fn type_info() -> &'static TypeInfo {
                static CELL: GenericTypeInfoCell = GenericTypeInfoCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    let fields = [
                        $(UnnamedField::new::<$name>($index),)*
                    ];
                    let info = TupleInfo::new::<Self>(&fields);
                    TypeInfo::Tuple(info)
                })
            }
        }

        impl<$($name: Reflect + Typed),*> Tuple for ($($name,)*) {
            #[inline]
            fn field(&self, index: usize) -> Option<&dyn Reflect> {
                match index {
                    $($index => Some(&self.$index as &dyn Reflect),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
                match index {
                    $($index => Some(&mut self.$index as &mut dyn Reflect),)*
                    _ => None,
                }
            }

            #[inline]
            fn field_len(&self) -> usize {
                $num
            }

            #[inline]
            fn iter_fields(&self) -> TupleFieldIter<'_> {
                TupleFieldIter::new(self)
            }

            #[inline]
            fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
                vec![
                    $(Box::new(self.$index),)*
                ]
            }
        }


        impl<$($name: Reflect + Typed),*> Reflect for ($($name,)*) {
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
                ReflectKind::Tuple
            }

            #[inline]
            fn reflect_ref(&self) -> ReflectRef<'_> {
                ReflectRef::Tuple(self)
            }

            #[inline]
            fn reflect_mut(&mut self) -> ReflectMut<'_> {
                ReflectMut::Tuple(self)
            }

            #[inline]
            fn reflect_owned(self: Box<Self>) -> ReflectOwned {
                ReflectOwned::Tuple(self)
            }

            #[inline]
            fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
                tuple_try_apply(self, value)
            }

            #[inline]
            fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
                tuple_partial_eq(self, other)
            }

            fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
                Ok(Box::new((
                    $(
                        self.$index.reflect_clone()?
                            .take::<$name>()
                            .expect("`Reflect::reflect_clone` should return the same type"),
                    )*
                )))
            }
        
            #[inline]
            fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                tuple_debug(self, f)
            }

            // Temporarily disable reflect_hash
        }



        impl<$($name: Reflect + Typed + GetTypeTraits),*> GetTypeTraits for ($($name,)*) {
            fn get_type_traits() -> TypeTraits {
                TypeTraits::of::<($($name,)*)>()
            }

            fn register_dependencies(_registry: &mut TypeRegistry) {
                $(_registry.register::<$name>();)*
            }
        }

        impl<$($name: FromReflect + Typed),*> FromReflect for ($($name,)*) {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let _ref_tuple = reflect.reflect_ref().as_tuple().ok()?;

                Some((
                    $(
                        <$name as FromReflect>::from_reflect(_ref_tuple.field($index)?)?,
                    )*
                ))
            }
        }
    };
}

range_invoke!(impl_reflect_tuple, 12: P);
