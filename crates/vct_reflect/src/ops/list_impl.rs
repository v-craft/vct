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
        TypeInfo, TypePath, MaybeTyped, ReflectKind, ListInfo,
    },
};

#[derive(Default)]
pub struct DynamicList {
    target_type: Option<&'static TypeInfo>,
    values: Vec<Box<dyn PartialReflect>>,
}

impl TypePath for DynamicList {
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicList"
    }
    fn short_type_path() -> &'static str {
        "DynamicList"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynamicList")
    }
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicList {
    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::List(_)),
                "expected TypeInfo::List but received: {target_type:?}"
            );
        }
        self.target_type = target_type;
    }

    #[inline]
    pub fn push_box(&mut self, value: Box<dyn PartialReflect>) {
        self.values.push(value);
    }

    #[inline]
    pub fn push<T: PartialReflect>(&mut self, value: T) {
        self.values.push(Box::new(value));
    }
}

impl PartialReflect for DynamicList {
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
        let other = value.reflect_ref().as_list()?;

        for (idx, other_item) in other.iter().enumerate() {
            if idx < self.len() {
                if let Some(item) = self.get_mut(idx) {
                    item.try_apply(other_item)?;
                }
            } else {
                List::push(self, other_item.to_dynamic());
            }
        }
        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::List
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::List(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::List(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::List(self)
    }

    fn reflect_hash(&self) -> Option<u64> {
        let mut hasher = reflect_hasher();

        TypeId::of::<Self>().hash(&mut hasher);

        self.len().hash(&mut hasher);

        for  val in self.iter() {
            hasher.write_u64(val.reflect_hash()?);
        }

        Some(hasher.finish())
    }

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        let ReflectRef::List(other) = other.reflect_ref() else {
            return Some(false);
        };

        if self.len() != other.len() {
            return Some(false);
        }

        for (self_val, other_val) in self.iter().zip(other.iter()) {
            let result = self_val.reflect_partial_eq(other_val);
            if result != Some(true) {
                return result;
            }
        }

        Some(true)
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicList(")?;
        list_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

}

impl MaybeTyped for DynamicList {}

impl fmt::Debug for DynamicList {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl<T: PartialReflect> FromIterator<T> for DynamicList {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        values
            .into_iter()
            .map(|field| Box::new(field).into_partial_reflect())
            .collect()
    }
}

impl FromIterator<Box<dyn PartialReflect>> for DynamicList {
    fn from_iter<I: IntoIterator<Item = Box<dyn PartialReflect>>>(values: I) -> Self {
        Self {
            target_type: None,
            values: values.into_iter().collect(),
        }
    }
}

impl IntoIterator for DynamicList {
    type Item = Box<dyn PartialReflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

pub trait List: PartialReflect {
    fn get(&self, index: usize) -> Option<&dyn PartialReflect>;

    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    fn insert(&mut self, index: usize, element: Box<dyn PartialReflect>);

    fn remove(&mut self, index: usize) -> Box<dyn PartialReflect>;

    #[inline]
    fn push(&mut self, value: Box<dyn PartialReflect>) {
        self.insert(self.len(), value);
    }

    #[inline]
    fn pop(&mut self) -> Option<Box<dyn PartialReflect>> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(self.len() - 1))
        }
    }

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> ListItemIter<'_>;

    fn drain(&mut self) -> Vec<Box<dyn PartialReflect>>;

    fn to_dynamic_list(&self) -> DynamicList {
        DynamicList {
            target_type: self.get_target_type_info(),
            values: self.iter().map(PartialReflect::to_dynamic).collect(),
        }
    }

    fn get_target_list_info(&self) -> Option<&'static ListInfo> {
        self.get_target_type_info()?.as_list().ok()
    }
}

pub struct ListItemIter<'a> {
    list: &'a dyn List,
    index: usize,
}

impl ListItemIter<'_> {
    #[inline(always)]
    pub fn new(list: &dyn List) -> ListItemIter<'_> {
        ListItemIter { list, index: 0 }
    }
}

impl<'a> Iterator for ListItemIter<'a> {
    type Item = &'a dyn PartialReflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.list.get(self.index);
        self.index += value.is_some() as usize;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.list.len();
        (size - self.index, Some(size))
    }
}

impl ExactSizeIterator for ListItemIter<'_> {}

impl List for DynamicList {
    #[inline]
    fn get(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.values.get(index).map(|value| &**value)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        self.values.get_mut(index).map(|value| &mut **value)
    }

    #[inline]
    fn insert(&mut self, index: usize, element: Box<dyn PartialReflect>) {
        self.values.insert(index, element);
    }

    #[inline]
    fn remove(&mut self, index: usize) -> Box<dyn PartialReflect> {
        self.values.remove(index)
    }

    #[inline]
    fn push(&mut self, value: Box<dyn PartialReflect>) {
        DynamicList::push_box(self, value);
    }

    #[inline]
    fn pop(&mut self) -> Option<Box<dyn PartialReflect>> {
        self.values.pop()
    }

    #[inline]
    fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn iter(&self) -> ListItemIter<'_> {
        ListItemIter::new(self)
    }

    #[inline]
    fn drain(&mut self) -> Vec<Box<dyn PartialReflect>> {
        self.values.drain(..).collect()
    }
}

pub fn list_partial_eq<L: List + ?Sized>(x: &L, y: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::List(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.len() != y.len() {
        return Some(false);
    }

    for (x_value, y_value) in x.iter().zip(y.iter()) {
        let result = x_value.reflect_partial_eq(y_value);
        if result != Some(true) {
            return result;
        }
    }

    Some(true)
}

pub fn list_debug(dyn_list: &dyn List, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_list();
    for item in dyn_list.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}

