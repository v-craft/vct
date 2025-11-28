use crate::{
    PartialReflect, Reflect,
    info::{ListInfo, MaybeTyped, ReflectKind, TypeInfo, TypePath},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
    reflect_hasher,
};
use alloc::{boxed::Box, vec::Vec};
use core::{
    any::TypeId,
    fmt,
    hash::{Hash, Hasher},
};

/// A list of reflected values.
#[derive(Default)]
pub struct DynamicList {
    target_type: Option<&'static TypeInfo>,
    values: Vec<Box<dyn PartialReflect>>,
}

impl TypePath for DynamicList {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicList"
    }

    #[inline]
    fn type_name() -> &'static str {
        "DynamicList"
    }

    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicList")
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

impl DynamicList {
    /// Sets the [`TypeInfo`] to be represented by this `DynamicList`.
    /// # Panics
    ///
    /// Panics if the given [`TypeInfo`] is not a [`TypeInfo::List`].
    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::List(_)),
                "expected TypeInfo::List but received: {target_type:?}"
            );
        }
        self.target_type = target_type;
    }

    /// Appends a [`Reflect`] trait object to the list.
    #[inline]
    pub fn push_box(&mut self, value: Box<dyn PartialReflect>) {
        self.values.push(value);
    }

    /// Appends a typed value to the list.
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

        for val in self.iter() {
            hasher.write_u64(val.reflect_hash()?);
        }

        Some(hasher.finish())
    }

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        // Not Inline: `list_partial_eq()` is inline always
        list_partial_eq(self, other)
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

impl<'a> IntoIterator for &'a DynamicList {
    type Item = &'a dyn PartialReflect;
    type IntoIter = ListItemIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A trait used to power [list-like] operations via [reflection].
///
/// This corresponds to types, like [`Vec`], which contain an ordered sequence
/// of elements that implement [`Reflect`].
pub trait List: PartialReflect {
    /// Returns a reference to the element at `index`, or `None` if out of bounds.
    fn get(&self, index: usize) -> Option<&dyn PartialReflect>;

    /// Returns a mutable reference to the element at `index`, or `None` if out of bounds.
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    /// Inserts an element at position `index` within the list,
    /// shifting all elements after it towards the back of the list.
    ///
    /// # Panics
    /// Panics if `index > len`.
    fn insert(&mut self, index: usize, element: Box<dyn PartialReflect>);

    /// Removes and returns the element at position `index` within the list,
    /// shifting all elements before it towards the front of the list.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    fn remove(&mut self, index: usize) -> Box<dyn PartialReflect>;

    /// Appends an element to the _back_ of the list.
    #[inline]
    fn push(&mut self, value: Box<dyn PartialReflect>) {
        self.insert(self.len(), value);
    }

    /// Removes the _back_ element from the list and returns it, or [`None`] if it is empty.
    #[inline]
    fn pop(&mut self) -> Option<Box<dyn PartialReflect>> {
        if self.is_empty() {
            None
        } else {
            Some(self.remove(self.len() - 1))
        }
    }

    /// Returns the number of elements in the list.
    fn len(&self) -> usize;

    /// Returns `true` if the collection contains no elements.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the list.
    fn iter(&self) -> ListItemIter<'_>;

    /// Drain the elements of this list to get a vector of owned values.
    ///
    /// After calling this function, `self` will be empty. The order of items in the returned
    /// [`Vec`] will match the order of items in `self`.
    fn drain(&mut self) -> Vec<Box<dyn PartialReflect>>;

    /// Creates a new [`DynamicList`] from this list.
    fn to_dynamic_list(&self) -> DynamicList {
        DynamicList {
            target_type: self.get_target_type_info(),
            values: self.iter().map(PartialReflect::to_dynamic).collect(),
        }
    }

    /// Will return `None` if [`TypeInfo`] is not available.
    #[inline]
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

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// It's `inline(always)`, Usually recommended only for impl `reflect_partial_eq`.
#[inline(always)]
pub fn list_partial_eq<L: List + ?Sized>(x: &L, y: &dyn PartialReflect) -> Option<bool> {
    // Inline: this function **should only** be used to impl `PartialReflect::reflect_partial_eq`
    // Compilation times is related to the quantity of type A.
    // Therefore, inline has no negative effects.
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

/// The default debug formatter for [`List`] types.
pub fn list_debug(dyn_list: &dyn List, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `PartialReflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_list();
    for item in dyn_list.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}
