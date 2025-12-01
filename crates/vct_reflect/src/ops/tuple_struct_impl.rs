use alloc::{boxed::Box, vec::Vec};
use core::fmt;

use crate::{
    PartialReflect, Reflect,
    info::{MaybeTyped, ReflectKind, TupleStructInfo, TypeInfo, TypePath},
    ops::{ApplyError, DynamicTuple, ReflectMut, ReflectOwned, ReflectRef, Tuple},
};

impl From<DynamicTuple> for DynamicTupleStruct {
    fn from(value: DynamicTuple) -> Self {
        Self {
            target_type: None,
            fields: Tuple::drain(Box::new(value)),
        }
    }
}

/// A tuple struct which allows fields to be added at runtime.
#[derive(Default)]
pub struct DynamicTupleStruct {
    target_type: Option<&'static TypeInfo>,
    fields: Vec<Box<dyn PartialReflect>>,
}

impl TypePath for DynamicTupleStruct {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicTupleStruct"
    }

    #[inline]
    fn type_name() -> &'static str {
        "DynamicTupleStruct"
    }

    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicTupleStruct")
    }

    #[inline]
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }

    #[inline]
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicTupleStruct {
    /// Sets the [`TypeInfo`] to be represented by this `DynamicTupleStruct`.
    ///
    /// # Panics
    ///
    /// Panics if the given [`TypeInfo`] is not a [`TypeInfo::TupleStruct`].
    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::TupleStruct(_)),
                "expected TypeInfo::TupleStruct but received: {target_type:?}"
            );
        }
        self.target_type = target_type;
    }

    /// Appends an element with value `value` to the tuple struct.
    #[inline]
    pub fn insert_boxed(&mut self, value: Box<dyn PartialReflect>) {
        self.fields.push(value);
    }

    /// Appends a typed element with value `value` to the tuple struct.
    #[inline]
    pub fn insert<T: PartialReflect>(&mut self, value: T) {
        self.fields.push(Box::new(value));
    }
}

impl PartialReflect for DynamicTupleStruct {
    #[inline]
    fn get_target_type_info(&self) -> Option<&'static TypeInfo> {
        self.target_type
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
        None
    }

    #[inline]
    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        None
    }

    #[inline]
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Err(self)
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        let other = value.reflect_ref().as_tuple_struct()?;

        for (idx, other_field) in other.iter_fields().enumerate() {
            if let Some(field) = self.field_mut(idx) {
                field.try_apply(other_field)?;
            }
        }
        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::TupleStruct
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::TupleStruct(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::TupleStruct(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::TupleStruct(self)
    }

    #[inline]
    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        tuple_struct_partial_eq(self, other)
    }

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicTupleStruct(")?;
        tuple_struct_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicTupleStruct {}

impl fmt::Debug for DynamicTupleStruct {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl FromIterator<Box<dyn PartialReflect>> for DynamicTupleStruct {
    fn from_iter<T: IntoIterator<Item = Box<dyn PartialReflect>>>(iter: T) -> Self {
        Self {
            target_type: None,
            fields: iter.into_iter().collect(),
        }
    }
}

impl IntoIterator for DynamicTupleStruct {
    type Item = Box<dyn PartialReflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicTupleStruct {
    type Item = &'a dyn PartialReflect;
    type IntoIter = TupleStructFieldIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_fields()
    }
}

/// A trait used to power [tuple struct-like] operations via [reflection].
///
/// This trait uses the [`Reflect`] trait to allow implementors to have their fields
/// be dynamically addressed by index.
pub trait TupleStruct: PartialReflect {
    /// Returns a reference to the value of the field with index `index` as a
    /// `&dyn Reflect`.
    fn field(&self, index: usize) -> Option<&dyn PartialReflect>;

    /// Returns a mutable reference to the value of the field with index `index`
    /// as a `&mut dyn Reflect`.
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    /// Returns the number of fields in the tuple struct.
    fn field_len(&self) -> usize;

    /// Returns an iterator over the values of the tuple struct's fields.
    fn iter_fields(&self) -> TupleStructFieldIter<'_>;

    /// Creates a new [`DynamicTupleStruct`] from this tuple struct.
    fn to_dynamic_tuple_struct(&self) -> DynamicTupleStruct {
        DynamicTupleStruct {
            target_type: self.get_target_type_info(),
            fields: self.iter_fields().map(PartialReflect::to_dynamic).collect(),
        }
    }

    /// Will return `None` if [`TypeInfo`] is not available.
    #[inline]
    fn get_target_tuple_struct_info(&self) -> Option<&'static TupleStructInfo> {
        self.get_target_type_info()?.as_tuple_struct().ok()
    }
}

/// An iterator over the field values of a tuple struct.
pub struct TupleStructFieldIter<'a> {
    tuple_struct: &'a dyn TupleStruct,
    index: usize,
}

impl<'a> TupleStructFieldIter<'a> {
    #[inline(always)]
    pub fn new(value: &'a dyn TupleStruct) -> Self {
        TupleStructFieldIter {
            tuple_struct: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for TupleStructFieldIter<'a> {
    type Item = &'a dyn PartialReflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple_struct.field(self.index);
        self.index += value.is_some() as usize;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.tuple_struct.field_len();
        (size - self.index, Some(size))
    }
}

impl<'a> ExactSizeIterator for TupleStructFieldIter<'a> {}

impl TupleStruct for DynamicTupleStruct {
    #[inline]
    fn field(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.fields.get(index).map(|field| &**field)
    }

    #[inline]
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        self.fields.get_mut(index).map(|field| &mut **field)
    }

    #[inline]
    fn field_len(&self) -> usize {
        self.fields.len()
    }

    #[inline]
    fn iter_fields(&self) -> TupleStructFieldIter<'_> {
        TupleStructFieldIter::new(self)
    }
}

/// A convenience trait which combines fetching and downcasting of tuple struct fields.
pub trait GetTupleStructField {
    /// Returns a reference to the value of the field with index `index`,
    /// downcast to `T`.
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T>;

    /// Returns a mutable reference to the value of the field with index
    /// `index`, downcast to `T`.
    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T>;
}

impl<S: TupleStruct> GetTupleStructField for S {
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.try_downcast_ref::<T>())
    }

    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.try_downcast_mut::<T>())
    }
}

impl GetTupleStructField for dyn TupleStruct {
    #[inline]
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.try_downcast_ref::<T>())
    }

    #[inline]
    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.try_downcast_mut::<T>())
    }
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn tuple_struct_partial_eq(x: &dyn TupleStruct, y: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::TupleStruct(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    for (idx, y_field) in y.iter_fields().enumerate() {
        if let Some(x_field) = x.field(idx) {
            let result = x_field.reflect_partial_eq(y_field);
            if result != Some(true) {
                return result;
            }
        } else {
            return Some(false);
        }
    }

    Some(true)
}

/// The default debug formatter for [`Tuple`] types.
/// 
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn tuple_struct_debug(
    dyn_tuple_struct: &dyn TupleStruct,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    // This function should only be used to impl `PartialReflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_tuple(
        dyn_tuple_struct
            .get_target_type_info()
            .map(TypeInfo::type_path)
            .unwrap_or("_"),
    );
    for field in dyn_tuple_struct.iter_fields() {
        debug.field(&field as &dyn fmt::Debug);
    }
    debug.finish()
}
