use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::{
    Reflect, cell::NonGenericTypeInfoCell, 
    info::{ArrayInfo, OpaqueInfo, ReflectKind, TypeInfo, TypePath, Typed}, 
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef}, 
    reflect::impl_cast_reflect_fn, reflect_hasher
};


/// Representing [`Array`]`, used to dynamically modify the type of data and information.
/// 
/// Dynamic types are special in that their TypeInfo is [`OpaqueInfo`], 
/// but other APIs are consistent with the type they represent, such as [`reflect_kind`], [`reflect_ref`]
/// 
/// [`reflect_kind`]: crate::Reflect::reflect_kind
/// [`reflect_ref`]: crate::Reflect::reflect_ref
pub struct DynamicArray {
    array_info: Option<&'static TypeInfo>, // Ensure it is None or ArrayInfo
    values: Box<[Box<dyn Reflect>]>,
}

impl TypePath for DynamicArray {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicArray"
    }
    #[inline]
    fn type_name() -> &'static str {
        "DynamicArray"
    }
    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicArray")
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

impl Typed for DynamicArray {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl DynamicArray {
    /// Creates a new [`DynamicArray`].
    #[inline]
    pub fn new(values: Box<[Box<dyn Reflect>]>) -> Self {
        Self {
            array_info: None,
            values,
        }
    }

    /// Sets the [`TypeInfo`] to be represented by this `DynamicArray`.
    /// 
    /// # Panic
    /// 
    /// If the input is not array info or None.
    #[inline]
    pub fn set_type_info(&mut self, array_info: Option<&'static TypeInfo>) {
        match array_info {
            Some(TypeInfo::Array(_)) | None => {},
            _ => { panic!("Call `DynamicArray::set_type_info`, but the input is not array information or None.") },
        }
        
        self.array_info = array_info;
    }
}

impl Reflect for DynamicArray {
    impl_cast_reflect_fn!();

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.array_info
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

    #[inline]
    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        array_try_apply(self, value)
    }

    #[inline]
    fn reflect_hash(&self) -> Option<u64> {
        array_hash(self)
    }

    #[inline]
    fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
        array_partial_eq(self, other)
    }

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicArray(")?;
        array_debug(self, f)?;
        write!(f, ")")
    }

    // `to_dynamic` needs to ensure that the new object is "completely dynamic" semantically, except for the Opaque type.
    // Therefore, use the default implementation directly.
}

impl fmt::Debug for DynamicArray {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl<T: Reflect> FromIterator<T> for DynamicArray {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        Self {
            array_info: None,
            values: values.into_iter().map(|value| Box::new(value).into_reflect()).collect(),
        }
    }
}

impl FromIterator<Box<dyn Reflect>> for DynamicArray {
    fn from_iter<I: IntoIterator<Item = Box<dyn Reflect>>>(values: I) -> Self {
        Self {
            array_info: None,
            values: values.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        }
    }
}

impl IntoIterator for DynamicArray {
    type Item = Box<dyn Reflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_vec().into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicArray {
    type Item = &'a dyn Reflect;
    type IntoIter = ArrayItemIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// A trait used to power [array-like] operations via [reflection].
///
/// This corresponds to true Rust arrays like `[T; N]`,
/// but also to any fixed-size linear sequence types.
/// It is expected that implementors of this trait uphold this contract
/// and maintain a fixed size as returned by the [`Array::len`] method.
pub trait Array: Reflect {
    /// Returns a reference to the element at `index`, or `None` if out of bounds.
    fn get(&self, index: usize) -> Option<&dyn Reflect>;

    /// Returns a mutable reference to the element at `index`, or `None` if out of bounds.
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    /// Returns the number of elements in the array.
    fn len(&self) -> usize;

    /// Returns `true` if the collection contains no elements.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the array.
    fn iter(&self) -> ArrayItemIter<'_>;

    /// Drain the elements of this array to get a vector of owned values.
    fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>>;

    /// Creates a new [`DynamicArray`] from this array.
    fn to_dynamic_array(&self) -> DynamicArray {
        DynamicArray {
            array_info: self.represented_type_info(),
            values: self.iter().map(Reflect::to_dynamic).collect(),
        }
    }

    /// Get actual [`ArrayInfo`] of underlying types.
    /// 
    /// If it is a dynamic type, it will return `None`.
    /// 
    /// If it is not a dynamic type and the returned value is not `None` or `ArrayInfo`, it will panic.
    /// (If you want to implement dynamic types yourself, please return None.)
    #[inline]
    fn reflect_array_info(&self) -> Option<&'static ArrayInfo> {
        self.reflect_type_info().as_array().ok()
    }

    /// Get the [`ArrayInfo`] of representation.
    /// 
    /// Normal types return their own information,
    /// while dynamic types return `None`` if they do not represent an object
    #[inline]
    fn represented_array_info(&self) -> Option<&'static ArrayInfo> {
        self.represented_type_info()?.as_array().ok()
    }
}

pub struct ArrayItemIter<'a> {
    array: &'a dyn Array,
    index: usize,
}

/// An iterator over an [`Array`].
impl ArrayItemIter<'_> {
    #[inline(always)]
    pub fn new(array: &dyn Array) -> ArrayItemIter<'_> {
        ArrayItemIter { array, index: 0 }
    }
}

impl<'a> Iterator for ArrayItemIter<'a> {
    type Item = &'a dyn Reflect;

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
    fn get(&self, index: usize) -> Option<&dyn Reflect> {
        self.values.get(index).map(|value| &**value)
    }

    #[inline]
    fn get_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
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
    fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
        self.values.into_vec()
    }

    #[inline]
    fn reflect_array_info(&self) -> Option<&'static ArrayInfo> {
        None
    }

    #[inline]
    fn represented_array_info(&self) -> Option<&'static ArrayInfo> {
        self.array_info?.as_array().ok()
    }

    // `to_dynamic` needs to ensure that the new object is "completely dynamic" semantically, except for the Opaque type.
    // Therefore, use the default implementation directly.
}

/// A function used to assist in the implementation of `reflect_try_apply`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn array_try_apply(x: &mut dyn Array, y: &dyn Reflect) ->Result<(), ApplyError> {
    let y = y.reflect_ref().as_array()?;

    if x.len() != y.len() {
        return Err(ApplyError::DifferentSize {
            from_size: y.len(),
            to_size: x.len(),
        });
    }

    for (idx, y_item) in y.iter().enumerate() {
        let item = x.get_mut(idx).unwrap();
        item.try_apply(y_item)?;
    }
    Ok(())
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn array_partial_eq(x: &dyn Array, y: &dyn Reflect) -> Option<bool> {
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

/// A function used to assist in the implementation of `reflect_hash`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn array_hash(x: &dyn Array) -> Option<u64> {
    let mut hasher = reflect_hasher();
    x.type_id().hash(&mut hasher);
    x.len().hash(&mut hasher);
    for value in x.iter() {
        hasher.write_u64(value.reflect_hash()?);
    }
    Some(hasher.finish())
}

/// A function used to assist in the implementation of `reflect_debug`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn array_debug(dyn_array: &dyn Array, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `Reflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_list();
    for item in dyn_array.iter() {
        debug.entry(&item as &dyn fmt::Debug);
    }
    debug.finish()
}
