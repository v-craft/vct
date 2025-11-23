use core::fmt;
use alloc::{
    vec::Vec, boxed::Box,
};
use crate::{
    PartialReflect, Reflect, 
    info::{
        ReflectKind, TypeInfo, TypePath, TupleInfo, MaybeTyped,
    }, 
    ops::{
        ApplyError, ReflectMut, ReflectOwned, ReflectRef
    }
};

#[derive(Default)]
pub struct DynamicTuple {
    target_type: Option<&'static TypeInfo>,
    fields: Vec<Box<dyn PartialReflect>>,
}

impl TypePath for DynamicTuple {
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicTuple"
    }
    fn short_type_path() -> &'static str {
        "DynamicTuple"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynamicTuple")
    }
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicTuple {
    pub fn set_target_type(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::Tuple(_)),
                "expected TypeInfo::Tuple but received: {target_type:?}"
            );
        }
        self.target_type = target_type;
    }

    #[inline]
    pub fn insert_boxed(&mut self, value: Box<dyn PartialReflect>) {
        self.target_type = None;
        self.fields.push(value);
    }

    #[inline]
    pub fn insert<T: PartialReflect>(&mut self, value: T) {
        self.target_type = None;
        self.fields.push(Box::new(value));
    }
}

impl PartialReflect for DynamicTuple {
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
        let other = value.reflect_ref().as_tuple()?;

        for (idx, other_field) in other.iter_fields().enumerate() {
            if let Some(field) = self.field_mut(idx) {
                field.try_apply(other_field)?;
            }
        }
        Ok(())
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

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        // 手动内联
        let ReflectRef::Tuple(other) = other.reflect_ref() else {
            return Some(false);
        };

        if self.field_len() != other.field_len() {
            return Some(false);
        }

        for (field, other_field) in self.iter_fields().zip(other.iter_fields()) {
            let result = field.reflect_partial_eq(other_field);
            if result != Some(true) {
                return result;
            }
        }
        Some(true)
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicTuple(")?;
        tuple_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicTuple{}

impl fmt::Debug for DynamicTuple {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl FromIterator<Box<dyn PartialReflect>> for DynamicTuple {
    fn from_iter<I: IntoIterator<Item = Box<dyn PartialReflect>>>(fields: I) -> Self {
        Self {
            target_type: None,
            fields: fields.into_iter().collect(),
        }
    }
}

impl IntoIterator for DynamicTuple {
    type Item = Box<dyn PartialReflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicTuple {
    type Item = &'a dyn PartialReflect;
    type IntoIter = TupleFieldIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_fields()
    }
}

pub trait Tuple: PartialReflect {
    /// Returns a reference to the value of the field with index `index` as a
    /// `&dyn Reflect`.
    fn field(&self, index: usize) -> Option<&dyn PartialReflect>;

    /// Returns a mutable reference to the value of the field with index `index`
    /// as a `&mut dyn Reflect`.
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    /// Returns the number of fields in the tuple.
    fn field_len(&self) -> usize;

    /// Returns an iterator over the values of the tuple's fields.
    fn iter_fields(&self) -> TupleFieldIter<'_>;

    /// Drain the fields of this tuple to get a vector of owned values.
    fn drain(self: Box<Self>) -> Vec<Box<dyn PartialReflect>>;

    /// Creates a new [`DynamicTuple`] from this tuple.
    fn to_dynamic_tuple(&self) -> DynamicTuple {
        DynamicTuple {
            target_type: self.get_target_type_info(),
            fields: self.iter_fields().map(PartialReflect::to_dynamic).collect(),
        }
    }

    /// Will return `None` if [`TypeInfo`] is not available.
    fn get_target_tuple_info(&self) -> Option<&'static TupleInfo> {
        self.get_target_type_info()?.as_tuple().ok()
    }
}

impl Tuple for DynamicTuple {
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
    fn iter_fields(&self) -> TupleFieldIter<'_> {
        TupleFieldIter::new(self)
    }

    #[inline]
    fn drain(self: Box<Self>) -> Vec<Box<dyn PartialReflect>> {
        self.fields
    }
}

pub struct TupleFieldIter<'a> {
    tuple: &'a dyn Tuple,
    index: usize,
}

impl<'a> TupleFieldIter<'a> {
    #[inline(always)]
    pub fn new(value: &'a dyn Tuple) -> Self {
        TupleFieldIter {
            tuple: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for TupleFieldIter<'a> {
    type Item = &'a dyn PartialReflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.tuple.field(self.index);
        self.index += value.is_some() as usize;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.tuple.field_len();
        (size - self.index, Some(size))
    }
}

impl<'a> ExactSizeIterator for TupleFieldIter<'a> {}

pub trait GetTupleField {
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T>;
    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T>;
}

impl<S: Tuple> GetTupleField for S {
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.try_downcast_ref::<T>())
    }

    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.try_downcast_mut::<T>())
    }
}

impl GetTupleField for dyn Tuple {
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

pub fn tuple_partial_eq<T: Tuple + ?Sized>(x: &T, y: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::Tuple(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    for (x_field, y_field) in x.iter_fields().zip(y.iter_fields()) {
        let result = x_field.reflect_partial_eq(y_field);
        if result != Some(true) {
            return result;
        }
    }
    Some(true)
}

pub fn tuple_debug(dyn_tuple: &dyn Tuple, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_tuple("");
    for field in dyn_tuple.iter_fields() {
        debug.field(&field as &dyn fmt::Debug);
    }
    debug.finish()
}
