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

use crate::{
    FromReflect, Reflect,
    cell::{GenericTypeInfoCell, GenericTypePathCell, NonGenericTypeInfoCell},
    info::{ReflectKind, TupleInfo, TypeInfo, TypePath, Typed, UnnamedField},
    ops::{
        ApplyError, ReflectCloneError, ReflectMut, ReflectOwned, ReflectRef, Tuple, TupleFieldIter,
        tuple_debug, tuple_partial_eq, tuple_try_apply, tuple_hash,
    },
    registry::{GetTypeTraits, TypeRegistry, TypeTraits, FromType, TypeTraitDefault, TypeTraitDeserialize, TypeTraitFromPtr, TypeTraitFromReflect, TypeTraitSerialize},
};
use alloc::{boxed::Box, vec, vec::Vec};
use core::fmt;
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
            #[inline]
            fn type_ident() -> &'static str {
                "()"
            }
        }
    };
    (1: [$zero:ident]) => {
        impl<$zero: TypePath> TypePath for ($zero,) {
            fn type_path() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    $crate::impls::concat(&["(" , $zero::type_path() , ",)"])
                })
            }

            fn type_name() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    $crate::impls::concat(&["(" , $zero::type_name() , ",)"])
                })
            }

            fn type_ident() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    $crate::impls::concat(&["(" , $zero::type_ident() , ",)"])
                })
            }
        }
    };
    ($_:literal: [$zero:ident, $($index:ident),*]) => {
        impl<$zero: TypePath, $($index: TypePath),*> TypePath for ($zero, $($index),*) {
            fn type_path() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    $crate::impls::concat(&["(", $zero::type_path() $(, ", ", $index::type_path())* , ")"])
                })
            }

            fn type_name() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    $crate::impls::concat(&["(", $zero::type_name() $(, ", ", $index::type_name())* , ")"])
                })
            }

            fn type_ident() -> &'static str {
                static CELL: GenericTypePathCell = GenericTypePathCell::new();
                CELL.get_or_insert::<Self, _>(|| {
                    $crate::impls::concat(&["(", $zero::type_ident() $(, ", ", $index::type_ident())* , ")"])
                })
            }
        }
    };
}

range_invoke!(impl_type_path_tuple, 12);

macro_rules! impl_reflect_tuple {
    (0: []) => {
        impl Typed for () {
            fn type_info() -> &'static TypeInfo {
                static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
                CELL.get_or_init(|| {
                    let info = TupleInfo::new::<Self>(&[]);
                    TypeInfo::Tuple(info)
                })
            }
        }

        impl Tuple for () {
            #[inline]
            fn field(&self, _index: usize) -> Option<&dyn Reflect> {
                None
            }

            #[inline]
            fn field_mut(&mut self, _index: usize) -> Option<&mut dyn Reflect> {
                None
            }

            #[inline]
            fn field_len(&self) -> usize {
                0
            }

            #[inline]
            fn iter_fields(&self) -> TupleFieldIter<'_> {
                TupleFieldIter::new(self)
            }

            #[inline]
            fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
                Vec::new()
            }
        }

        impl Reflect for () {
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
                if value.is::<Self>() {
                    Ok(())
                } else {
                    Err(value)
                }
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
                if other.is::<Self>() {
                    Some(true)
                } else {
                    None
                }
            }

            fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
                Ok(Box::new(()))
            }

            #[inline]
            fn reflect_hash(&self) -> Option<u64> {
                let mut hasher = crate::reflect_hasher();
                <() as core::hash::Hash>::hash(self, &mut hasher);
                Some(core::hash::Hasher::finish(&hasher))
            }

            #[inline]
            fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Debug::fmt(self, f)
            }
        }
    
        impl GetTypeTraits for () {
            fn get_type_traits() -> TypeTraits {
                let mut type_traits = TypeTraits::of::<Self>();
                type_traits.insert::<TypeTraitDefault>(FromType::<Self>::from_type());
                type_traits.insert::<TypeTraitFromPtr>(FromType::<Self>::from_type());
                type_traits.insert::<TypeTraitFromReflect>(FromType::<Self>::from_type());
                type_traits.insert::<TypeTraitSerialize>(FromType::<Self>::from_type());
                type_traits.insert::<TypeTraitDeserialize>(FromType::<Self>::from_type());
                type_traits
            }
        }

        impl FromReflect for () {
            fn from_reflect(reflect: &dyn Reflect) -> Option<Self> {
                let _ref_tuple = reflect.reflect_ref().as_tuple().ok()?;

                Some(())
            }
        }
    };
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
            fn reflect_hash(&self) -> Option<u64> {
                tuple_hash(self)
            }

            #[inline]
            fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                tuple_debug(self, f)
            }
        }

        impl<$($name: Reflect + Typed + GetTypeTraits),*> GetTypeTraits for ($($name,)*) {
            fn get_type_traits() -> TypeTraits {
                TypeTraits::of::<($($name,)*)>()
            }

            fn register_dependencies(_registry: &mut TypeRegistry) {
                _registry.register::<()>();
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

