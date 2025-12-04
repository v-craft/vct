use crate::{
    Reflect,
    cell::NonGenericTypeInfoCell,
    info::{OpaqueInfo, ReflectKind, TupleInfo, TypeInfo, TypePath, Typed},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
    reflect::impl_cast_reflect_fn,
};
use alloc::{boxed::Box, vec::Vec};
use core::fmt;

/// Representing [`Tuple`]`, used to dynamically modify the type of data and information.
///
/// Dynamic types are special in that their TypeInfo is [`OpaqueInfo`],
/// but other APIs are consistent with the type they represent, such as [`reflect_kind`], [`reflect_ref`]
///
/// [`reflect_kind`]: crate::Reflect::reflect_kind
/// [`reflect_ref`]: crate::Reflect::reflect_ref
#[derive(Default)]
pub struct DynamicTuple {
    tuple_info: Option<&'static TypeInfo>,
    fields: Vec<Box<dyn Reflect>>,
}

impl TypePath for DynamicTuple {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicTuple"
    }

    #[inline]
    fn type_name() -> &'static str {
        "DynamicTuple"
    }

    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicTuple")
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

impl Typed for DynamicTuple {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl DynamicTuple {
    #[inline]
    pub const fn new() -> Self {
        Self {
            tuple_info: None,
            fields: Vec::new(),
        }
    }

    /// Sets the [`TypeInfo`] to be represented by this `DynamicTuple`.
    ///
    /// # Panic
    ///
    /// If the input is not list info or None.
    #[inline]
    pub fn set_type_info(&mut self, tuple_info: Option<&'static TypeInfo>) {
        match tuple_info {
            Some(TypeInfo::Tuple(_)) | None => {}
            _ => {
                panic!(
                    "Call `DynamicMap::set_type_info`, but the input is not tuple information or None."
                )
            }
        }

        self.tuple_info = tuple_info;
    }

    /// Appends an element with value `value` to the tuple.
    #[inline]
    pub fn insert_boxed(&mut self, value: Box<dyn Reflect>) {
        self.tuple_info = None;
        self.fields.push(value);
    }

    /// Appends a typed element with value `value` to the tuple.
    #[inline]
    pub fn insert<T: Reflect>(&mut self, value: T) {
        self.tuple_info = None;
        self.fields.push(Box::new(value));
    }
}

impl Reflect for DynamicTuple {
    impl_cast_reflect_fn!();

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.tuple_info
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

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicTuple(")?;
        tuple_debug(self, f)?;
        write!(f, ")")
    }
}

impl fmt::Debug for DynamicTuple {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl FromIterator<Box<dyn Reflect>> for DynamicTuple {
    fn from_iter<I: IntoIterator<Item = Box<dyn Reflect>>>(fields: I) -> Self {
        Self {
            tuple_info: None,
            fields: fields.into_iter().collect(),
        }
    }
}

impl IntoIterator for DynamicTuple {
    type Item = Box<dyn Reflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicTuple {
    type Item = &'a dyn Reflect;
    type IntoIter = TupleFieldIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_fields()
    }
}

/// A trait used to power [tuple-like] operations via [reflection].
///
/// This trait uses the [`Reflect`] trait to allow implementors to have their fields
/// be dynamically addressed by index.
pub trait Tuple: Reflect {
    /// Returns a reference to the value of the field with index `index` as a
    /// `&dyn Reflect`.
    fn field(&self, index: usize) -> Option<&dyn Reflect>;

    /// Returns a mutable reference to the value of the field with index `index`
    /// as a `&mut dyn Reflect`.
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    /// Returns the number of fields in the tuple.
    fn field_len(&self) -> usize;

    /// Returns an iterator over the values of the tuple's fields.
    fn iter_fields(&self) -> TupleFieldIter<'_>;

    /// Drain the fields of this tuple to get a vector of owned values.
    fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>>;

    /// Creates a new [`DynamicTuple`] from this tuple.
    fn to_dynamic_tuple(&self) -> DynamicTuple {
        DynamicTuple {
            tuple_info: self.represented_type_info(),
            fields: self.iter_fields().map(Reflect::to_dynamic).collect(),
        }
    }

    /// Get actual [`TupleInfo`] of underlying types.
    ///
    /// If it is a dynamic type, it will return `None`.
    ///
    /// If it is not a dynamic type and the returned value is not `None` or `TupleInfo`, it will panic.
    /// (If you want to implement dynamic types yourself, please return None.)
    #[inline]
    fn reflect_tuple_info(&self) -> Option<&'static TupleInfo> {
        self.reflect_type_info().as_tuple().ok()
    }

    /// Get the [`TupleInfo`] of representation.
    ///
    /// Normal types return their own information,
    /// while dynamic types return `None`` if they do not represent an object
    #[inline]
    fn represented_tuple_info(&self) -> Option<&'static TupleInfo> {
        self.represented_type_info()?.as_tuple().ok()
    }
}

impl Tuple for DynamicTuple {
    #[inline]
    fn field(&self, index: usize) -> Option<&dyn Reflect> {
        self.fields.get(index).map(|field| &**field)
    }

    #[inline]
    fn field_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
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
    fn drain(self: Box<Self>) -> Vec<Box<dyn Reflect>> {
        self.fields
    }

    #[inline]
    fn reflect_tuple_info(&self) -> Option<&'static TupleInfo> {
        None
    }

    #[inline]
    fn represented_tuple_info(&self) -> Option<&'static TupleInfo> {
        self.tuple_info?.as_tuple().ok()
    }
}

/// An iterator over the field values of a tuple.
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
    type Item = &'a dyn Reflect;

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

/// A convenience trait which combines fetching and downcasting of tuple fields.
pub trait GetTupleField {
    /// Returns a reference to the value of the field with index `index`,
    /// downcast to `T`.
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T>;

    /// Returns a mutable reference to the value of the field with index
    /// `index`, downcast to `T`.
    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T>;
}

impl<S: Tuple> GetTupleField for S {
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

impl GetTupleField for dyn Tuple {
    #[inline]
    fn get_field<T: Reflect>(&self, index: usize) -> Option<&T> {
        self.field(index)
            .and_then(|value| value.downcast_ref::<T>())
    }

    #[inline]
    fn get_field_mut<T: Reflect>(&mut self, index: usize) -> Option<&mut T> {
        self.field_mut(index)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

/// A function used to assist in the implementation of `try_apply`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn tuple_try_apply(x: &mut dyn Tuple, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_tuple()?;

    for (idx, y_field) in y.iter_fields().enumerate() {
        if let Some(field) = x.field_mut(idx) {
            field.try_apply(y_field)?;
        }
    }

    Ok(())
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn tuple_partial_eq(x: &dyn Tuple, y: &dyn Reflect) -> Option<bool> {
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

/// The default debug formatter for [`Tuple`] types.
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn tuple_debug(dyn_tuple: &dyn Tuple, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `PartialReflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_tuple("");
    for field in dyn_tuple.iter_fields() {
        debug.field(&field as &dyn fmt::Debug);
    }
    debug.finish()
}
