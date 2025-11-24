use core::{
    fmt,
    any::TypeId,
    hash::{Hash, Hasher},
};
use alloc::{
    vec::Vec,
    boxed::Box,
};
use crate::{
    PartialReflect, Reflect, reflect_hasher,
    ops::{
        ReflectRef, ReflectMut, ReflectOwned, ApplyError,
    },
    info::{
        TypeInfo, TypePath, MaybeTyped, ReflectKind, ArrayInfo,
    },
};

// Not impl Default: The length of Array needs to be determined.
pub struct DynamicArray {
    target_type: Option<&'static TypeInfo>,
    values: Box<[Box<dyn PartialReflect>]>,
}

impl TypePath for DynamicArray {
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicArray"
    }

    fn short_type_path() -> &'static str {
        "DynamicArray"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynamicArray")
    }
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicArray {
    #[inline]
    pub fn new(values: Box<[Box<dyn PartialReflect>]>) -> Self {
        Self {
            target_type: None,
            values,
        }
    }

    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::Array(_)),
                "expected TypeInfo::Array but received: {target_type:?}"
            );
        }

        self.target_type = target_type;
    }
}

impl PartialReflect for DynamicArray {
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
        let other = value.reflect_ref().as_array()?;

        if self.len() != other.len() {
            return Err(ApplyError::DifferentSize {
                from_size: other.len(),
                to_size: self.len(),
            });
        }

        for (idx, other_item) in other.iter().enumerate() {
            let item = self.get_mut(idx).unwrap();
            item.try_apply(other_item)?;
        }
        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Array
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Array(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Array(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Array(self)
    }

    fn reflect_hash(&self) -> Option<u64> {
        let mut hasher = reflect_hasher();
        TypeId::of::<Self>().hash(&mut hasher);
        self.len().hash(&mut hasher);
        for value in self.iter() {
            hasher.write_u64(value.reflect_hash()?);
        }
        Some(hasher.finish())
    }

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        let ReflectRef::Array(other) = other.reflect_ref() else {
            return Some(false);
        };

        if self.len() != other.len() {
            return Some(false);
        }

        for (item, other_item) in self.iter().zip(other.iter()) {
            let result = item.reflect_partial_eq(other_item);
            if result != Some(true) {
                return Some(false);
            }
        }

        Some(true)
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicArray(")?;
        array_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicArray {}

impl fmt::Debug for DynamicArray {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl<T: PartialReflect> FromIterator<T> for DynamicArray {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        values
            .into_iter()
            .map(|value| Box::new(value).into_partial_reflect())
            .collect()
    }
}

impl FromIterator<Box<dyn PartialReflect>> for DynamicArray {
    fn from_iter<I: IntoIterator<Item = Box<dyn PartialReflect>>>(values: I) -> Self {
        Self {
            target_type: None,
            values: values.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        }
    }
}

impl IntoIterator for DynamicArray {
    type Item = Box<dyn PartialReflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicArray {
    type Item = &'a dyn PartialReflect;
    type IntoIter = ArrayItemIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub trait Array: PartialReflect {
    fn get(&self, index: usize) -> Option<&dyn PartialReflect>;
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;
    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> ArrayItemIter<'_>;
    fn drain(self: Box<Self>) -> Vec<Box<dyn PartialReflect>>;
    fn to_dynamic_array(&self) -> DynamicArray {
        DynamicArray {
            target_type: self.get_target_type_info(),
            values: self.iter().map(PartialReflect::to_dynamic).collect(),
        }
    }
    fn get_target_array_info(&self) -> Option<&'static ArrayInfo> {
        self.get_target_type_info()?.as_array().ok()
    }
}

pub struct ArrayItemIter<'a> {
    array: &'a dyn Array,
    index: usize,
}

impl ArrayItemIter<'_> {
    #[inline(always)]
    pub fn new(array: &dyn Array) -> ArrayItemIter<'_> {
        ArrayItemIter { array, index: 0 }
    }
}

impl<'a> Iterator for ArrayItemIter<'a> {
    type Item = &'a dyn PartialReflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.array.get(self.index);
        self.index += value.is_some() as usize;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.array.len();
        (size - self.index, Some(size))
    }
}

impl<'a> ExactSizeIterator for ArrayItemIter<'a> {}

impl Array for DynamicArray {
    #[inline]
    fn get(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.values.get(index).map(|value| &**value)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        self.values.get_mut(index).map(|value| &mut **value)
    }

    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn iter(&self) -> ArrayItemIter<'_> {
        ArrayItemIter::new(self)
    }

    #[inline]
    fn drain(self: Box<Self>) -> Vec<Box<dyn PartialReflect>> {
        self.values.into_vec()
    }
}


pub fn array_partial_eq<A: Array + ?Sized>(x: &A, y: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::Array(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.len() != y.len() {
        return Some(false);
    }

    for (item, y_item) in x.iter().zip(y.iter()) {
        let result = item.reflect_partial_eq(y_item);
        if result != Some(true) {
            return Some(false);
        }
    }

    Some(true)
}

pub fn array_debug(dyn_array: &dyn Array, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_list();
    for item in dyn_array.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}

