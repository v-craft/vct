use crate::{
    Reflect,
    cell::NonGenericTypeInfoCell,
    info::{ListInfo, ReflectKind, TypeInfo, TypePath, Typed, OpaqueInfo},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
    reflect::impl_cast_reflect_fn,
    reflect_hasher,
};
use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt,
    hash::{Hash, Hasher},
};

/// Representing [`List`], used to dynamically modify the type of data and information.
/// 
/// Dynamic types are special in that their TypeInfo is [`OpaqueInfo`], 
/// but other APIs are consistent with the type they represent, such as [`reflect_kind`], [`reflect_ref`]
/// 
/// [`reflect_kind`]: crate::Reflect::reflect_kind
/// [`reflect_ref`]: crate::Reflect::reflect_ref
#[derive(Default)]
pub struct DynamicList {
    list_info: Option<&'static TypeInfo>,
    values: Vec<Box<dyn Reflect>>,
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

impl Typed for DynamicList {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl DynamicList {
    #[inline]
    pub const fn new() -> Self {
        Self { list_info: None, values: Vec::new() }
    }
    /// Sets the [`TypeInfo`] to be represented by this `DynamicList`.
    /// 
    /// # Panic
    /// 
    /// If the input is not list info or None.
    #[inline]
    pub fn set_type_info(&mut self, list_info: Option<&'static TypeInfo>) {
        match list_info {
            Some(TypeInfo::List(_)) | None => {},
            _ => { panic!("Call `DynamicList::set_type_info`, but the input is not list information or None.") },
        }

        self.list_info = list_info;
    }

    /// Appends a [`Reflect`] trait object to the list.
    #[inline]
    pub fn push_box(&mut self, value: Box<dyn Reflect>) {
        self.values.push(value);
    }

    /// Appends a typed value to the list.
    #[inline]
    pub fn push<T: Reflect>(&mut self, value: T) {
        self.values.push(Box::new(value));
    }
}

impl Reflect for DynamicList {
    impl_cast_reflect_fn!();

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.list_info
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

    #[inline]
    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        list_try_apply(self, value)
    }

    #[inline]
    fn reflect_hash(&self) -> Option<u64> {
        list_hash(self)
    }

    #[inline]
    fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
        list_partial_eq(self, other)
    }

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicList(")?;
        list_debug(self, f)?;
        write!(f, ")")
    }
}


impl fmt::Debug for DynamicList {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl<T: Reflect> FromIterator<T> for DynamicList {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self {
            list_info: None,
            values: values.into_iter().map(|field| Box::new(field).into_reflect()).collect()
        }
    }
}

impl FromIterator<Box<dyn Reflect>> for DynamicList {
    fn from_iter<I: IntoIterator<Item = Box<dyn Reflect>>>(values: I) -> Self {
        Self {
            list_info: None,
            values: values.into_iter().collect(),
        }
    }
}

impl IntoIterator for DynamicList {
    type Item = Box<dyn Reflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicList {
    type Item = &'a dyn Reflect;
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
pub trait List: Reflect {
    /// Returns a reference to the element at `index`, or `None` if out of bounds.
    fn get(&self, index: usize) -> Option<&dyn Reflect>;

    /// Returns a mutable reference to the element at `index`, or `None` if out of bounds.
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    /// Inserts an element at position `index` within the list,
    /// shifting all elements after it towards the back of the list.
    ///
    /// # Panics
    /// Panics if `index > len`.
    fn insert(&mut self, index: usize, element: Box<dyn Reflect>);

    /// Removes and returns the element at position `index` within the list,
    /// shifting all elements before it towards the front of the list.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    fn remove(&mut self, index: usize) -> Box<dyn Reflect>;

    /// Appends an element to the _back_ of the list.
    #[inline]
    fn push(&mut self, value: Box<dyn Reflect>) {
        self.insert(self.len(), value);
    }

    /// Removes the _back_ element from the list and returns it, or [`None`] if it is empty.
    #[inline]
    fn pop(&mut self) -> Option<Box<dyn Reflect>> {
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
    fn drain(&mut self) -> Vec<Box<dyn Reflect>>;

    /// Creates a new [`DynamicList`] from this list.
    /// 
    /// This function will replace all content with dynamic types, except for `Opaque`.
    fn to_dynamic_list(&self) -> DynamicList {
        DynamicList {
            list_info: self.represented_type_info(),
            values: self.iter().map(Reflect::to_dynamic).collect(),
        }
    }

    /// Get actual [`ListInfo`] of underlying types.
    /// 
    /// If it is a dynamic type, it will return `None`.
    /// 
    /// If it is not a dynamic type and the returned value is not `None` or `ListInfo`, it will panic.
    /// (If you want to implement dynamic types yourself, please return None.)
    #[inline]
    fn reflect_list_info(&self) -> Option<&'static ListInfo> {
        self.reflect_type_info().as_list().ok()
    }

    /// Get the [`ListInfo`] of representation.
    /// 
    /// Normal types return their own information,
    /// while dynamic types return `None`` if they do not represent an object
    #[inline]
    fn represented_list_info(&self) -> Option<&'static ListInfo> {
        self.represented_type_info()?.as_list().ok()
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
    type Item = &'a dyn Reflect;

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
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.values.get(index).map(|value| &**value)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.values.get_mut(index).map(|value| &mut **value)
    }

    #[inline]
    fn insert(&mut self, index: usize, element: Box<dyn Reflect>) {
        self.values.insert(index, element);
    }

    #[inline]
    fn remove(&mut self, index: usize) -> Box<dyn Reflect> {
        self.values.remove(index)
    }

    #[inline]
    fn push(&mut self, value: Box<dyn Reflect>) {
        DynamicList::push_box(self, value);
    }

    #[inline]
    fn pop(&mut self) -> Option<Box<dyn Reflect>> {
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
    fn drain(&mut self) -> Vec<Box<dyn Reflect>> {
        self.values.drain(..).collect()
    }

    #[inline]
    fn reflect_list_info(&self) -> Option<&'static ListInfo> {
        None
    }

    #[inline]
    fn represented_list_info(&self) -> Option<&'static ListInfo> {
        self.list_info?.as_list().ok()
    }
}



/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn list_try_apply(x: &mut dyn List, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_list()?;

    for (idx, y_item) in y.iter().enumerate() {
        if idx < x.len() {
            if let Some(item) = x.get_mut(idx) {
                item.try_apply(y_item)?;
            }
        } else {
            x.push(y_item.to_dynamic());
        }
    }
    Ok(())
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn list_partial_eq(x: &dyn List, y: &dyn Reflect) -> Option<bool> {
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

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn list_hash(x: &dyn List) -> Option<u64> {
    let mut hasher = reflect_hasher();

    x.type_id().hash(&mut hasher);
    x.len().hash(&mut hasher);
    for val in x.iter() {
        hasher.write_u64(val.reflect_hash()?);
    }

    Some(hasher.finish())
}

/// The default debug formatter for [`List`] types.
/// 
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn list_debug(dyn_list: &dyn List, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_list();
    for item in dyn_list.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}
