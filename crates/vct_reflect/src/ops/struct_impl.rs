use crate::{
    Reflect, reflect::impl_cast_reflect_fn,
    cell::NonGenericTypeInfoCell,
    info::{ReflectKind, StructInfo, TypeInfo, TypePath, Typed, OpaqueInfo},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
};
use alloc::{borrow::Cow, boxed::Box, string::ToString, vec::Vec};
use core::fmt;
use vct_utils::collections::HashMap;

/// Representing [`Struct`]`, used to dynamically modify the type of data and information.
/// 
/// Dynamic types are special in that their TypeInfo is [`OpaqueInfo`], 
/// but other APIs are consistent with the type they represent, such as [`reflect_kind`], [`reflect_ref`]
/// 
/// [`reflect_kind`]: crate::Reflect::reflect_kind
/// [`reflect_ref`]: crate::Reflect::reflect_ref
#[derive(Default)]
pub struct DynamicStruct {
    struct_info: Option<&'static TypeInfo>,
    fields: Vec<Box<dyn Reflect>>,
    field_names: Vec<Cow<'static, str>>,
    field_indices: HashMap<Cow<'static, str>, usize>,
}

impl TypePath for DynamicStruct {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicStruct"
    }

    #[inline]
    fn type_name() -> &'static str {
        "DynamicStruct"
    }

    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicStruct")
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

impl Typed for DynamicStruct {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl DynamicStruct {
    #[inline]
    pub const fn new() -> Self {
        Self {
            struct_info: None, 
            fields: Vec::new(), 
            field_names: Vec::new(), 
            field_indices: HashMap::<_,_>::new()
        }
    }
    /// Sets the [`StructInfo`] to be represented by this `DynamicStruct`.
    #[inline]
    pub fn set_type_info(&mut self, struct_info: Option<&'static TypeInfo>) {
        match struct_info {
            Some(TypeInfo::Struct(_)) | None => {},
            _ => { panic!("Call `DynamicStruct::set_type_info`, but the input is not struct information or None.") },
        }

        self.struct_info = struct_info;
    }

    /// Inserts a field named `name` with value `value` into the struct.
    ///
    /// If the field already exists, it is overwritten.
    pub fn insert_boxed(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        value: Box<dyn Reflect>,
    ) {
        let name: Cow<'static, str> = name.into();
        if let Some(index) = self.field_indices.get(&name) {
            self.fields[*index] = value;
        } else {
            self.fields.push(value);
            self.field_indices
                .insert(name.clone(), self.fields.len() - 1);
            self.field_names.push(name);
        }
    }

    /// Inserts a field named `name` with the typed value `value` into the struct.
    ///
    /// If the field already exists, it is overwritten.
    #[inline]
    pub fn insert<'a, T: Reflect>(&mut self, name: impl Into<Cow<'static, str>>, value: T) {
        self.insert_boxed(name, Box::new(value));
    }

    /// Gets the index of the field with the given name.
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.field_indices.get(name).copied()
    }
}

impl Reflect for DynamicStruct {
    impl_cast_reflect_fn!();

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.struct_info
    }

    #[inline]
    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        struct_try_apply(self, value)
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Struct
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Struct(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Struct(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Struct(self)
    }

    #[inline]
    fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
        struct_partial_eq(self, other)
    }

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicStruct(")?;
        struct_debug(self, f)?;
        write!(f, ")")
    }

}

impl fmt::Debug for DynamicStruct {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl<'a, N: Into<Cow<'static, str>>> FromIterator<(N, Box<dyn Reflect>)> for DynamicStruct {
    fn from_iter<T: IntoIterator<Item = (N, Box<dyn Reflect>)>>(fields: T) -> Self {
        let mut dynamic_struct = Self::default();
        for (name, value) in fields.into_iter() {
            dynamic_struct.insert_boxed(name, value);
        }
        dynamic_struct
    }
}

impl IntoIterator for DynamicStruct {
    type Item = Box<dyn Reflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicStruct {
    type Item = &'a dyn Reflect;
    type IntoIter = StructFieldIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_fields()
    }
}

/// A trait used to power [struct-like] operations via [reflection].(Including unit struct)
///
/// This trait uses the [`Reflect`] trait to allow implementors to have their fields
/// be dynamically addressed by both name and index.
pub trait Struct: Reflect {
    /// Returns a reference to the value of the field named `name` as a `&dyn
    /// PartialReflect`.
    fn field(&self, name: &str) -> Option<&dyn Reflect>;

    /// Returns a mutable reference to the value of the field named `name` as a
    /// `&mut dyn PartialReflect`.
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect>;

    /// Returns a reference to the value of the field with index `index` as a
    /// `&dyn PartialReflect`.
    fn field_at(&self, index: usize) -> Option<&dyn Reflect>;

    /// Returns a mutable reference to the value of the field with index `index`
    /// as a `&mut dyn PartialReflect`.
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect>;

    /// Returns the name of the field with index `index`.
    fn name_at(&self, index: usize) -> Option<&str>;

    /// Returns the number of fields in the struct.
    fn field_len(&self) -> usize;

    /// Returns an iterator over the values of the reflectable fields for this struct.
    fn iter_fields(&self) -> StructFieldIter<'_>;

    /// Creates a new [`DynamicStruct`] from this struct.
    fn to_dynamic_struct(&self) -> DynamicStruct {
        let mut dynamic_struct = DynamicStruct::default();
        dynamic_struct.set_type_info(self.represented_type_info());
        for (i, val) in self.iter_fields().enumerate() {
            dynamic_struct.insert_boxed(self.name_at(i).unwrap().to_string(), val.to_dynamic());
        }
        dynamic_struct
    }

    /// Get actual [`StructInfo`] of underlying types.
    /// 
    /// If it is a dynamic type, it will return `None`.
    /// 
    /// If it is not a dynamic type and the returned value is not `None` or `StructInfo`, it will panic.
    /// (If you want to implement dynamic types yourself, please return None.)
    #[inline]
    fn reflect_struct_info(&self) -> Option<&'static StructInfo> {
        self.reflect_type_info().as_struct().ok()
    }

    /// Get the [`StructInfo`] of representation.
    /// 
    /// Normal types return their own information,
    /// while dynamic types return `None`` if they do not represent an object
    #[inline]
    fn represented_struct_info(&self) -> Option<&'static StructInfo> {
        self.represented_type_info()?.as_struct().ok()
    }
}

/// An iterator over the field values of a struct.
pub struct StructFieldIter<'a> {
    struct_val: &'a dyn Struct,
    index: usize,
}

impl<'a> StructFieldIter<'a> {
    #[inline(always)]
    pub fn new(value: &'a dyn Struct) -> Self {
        StructFieldIter {
            struct_val: value,
            index: 0,
        }
    }
}

impl<'a> Iterator for StructFieldIter<'a> {
    type Item = &'a dyn Reflect;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let value = self.struct_val.field_at(self.index);
        self.index += value.is_some() as usize;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.struct_val.field_len();
        (size - self.index, Some(size))
    }
}

impl<'a> ExactSizeIterator for StructFieldIter<'a> {}

impl Struct for DynamicStruct {
    #[inline]
    fn field(&self, name: &str) -> Option<&dyn Reflect> {
        self.field_indices
            .get(name)
            .map(|index| &*self.fields[*index])
    }

    #[inline]
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn Reflect> {
        self.field_indices
            .get(name)
            .map(|index| &mut *self.fields[*index])
    }

    #[inline]
    fn field_at(&self, index: usize) -> Option<&dyn Reflect> {
        self.fields.get(index).map(|value| &**value)
    }

    #[inline]
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn Reflect> {
        self.fields.get_mut(index).map(|value| &mut **value)
    }

    #[inline]
    fn name_at(&self, index: usize) -> Option<&str> {
        self.field_names.get(index).map(AsRef::as_ref)
    }

    #[inline]
    fn field_len(&self) -> usize {
        self.fields.len()
    }

    #[inline]
    fn iter_fields(&self) -> StructFieldIter<'_> {
        StructFieldIter::new(self)
    }

    fn to_dynamic_struct(&self) -> DynamicStruct {
        DynamicStruct {
            struct_info: self.represented_type_info(),
            fields: self.fields.iter().map(|val| val.to_dynamic()).collect(),
            field_names: self.field_names.clone(),
            field_indices: self.field_indices.clone(),
        }
    }
}

/// A convenience trait which combines fetching and downcasting of struct fields.
pub trait GetStructField {
    /// Returns a reference to the value of the field named `name`,
    /// downcast to `T`.
    fn get_field<T: Reflect>(&self, name: &str) -> Option<&T>;

    /// Returns a mutable reference to the value of the field named `name`,
    /// downcast to `T`.
    fn get_field_mut<T: Reflect>(&mut self, name: &str) -> Option<&mut T>;
}

impl<S: Struct> GetStructField for S {
    fn get_field<T: Reflect>(&self, name: &str) -> Option<&T> {
        self.field(name)
            .and_then(|value| value.downcast_ref::<T>())
    }

    fn get_field_mut<T: Reflect>(&mut self, name: &str) -> Option<&mut T> {
        self.field_mut(name)
            .and_then(|value| value.downcast_mut::<T>())
    }
}

impl GetStructField for dyn Struct {
    #[inline]
    fn get_field<T: Reflect>(&self, name: &str) -> Option<&T> {
        self.field(name)
            .and_then(|value| value.downcast_ref::<T>())
    }

    #[inline]
    fn get_field_mut<T: Reflect>(&mut self, name: &str) -> Option<&mut T> {
        self.field_mut(name)
            .and_then(|value| value.downcast_mut::<T>())
    }
}


/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn struct_try_apply(x: &mut dyn Struct, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_struct()?;

    for (idx, y_field) in y.iter_fields().enumerate() {
        let name = y.name_at(idx).unwrap();
        if let Some(field) = x.field_mut(name) {
            field.try_apply(y_field)?;
        }
    }
    Ok(())
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn struct_partial_eq(x: &dyn Struct, y: &dyn Reflect) -> Option<bool> {
    let ReflectRef::Struct(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.field_len() != y.field_len() {
        return Some(false);
    }

    for (idx, y_field) in y.iter_fields().enumerate() {
        let name = y.name_at(idx).unwrap();
        if let Some(x_field) = x.field(name) {
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

/// The default debug formatter for [`Struct`] types.
/// 
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn struct_debug(dyn_struct: &dyn Struct, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `PartialReflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_struct(
        dyn_struct
            .represented_type_info()
            .map(TypeInfo::type_path)
            .unwrap_or("_"),
    );
    for field_index in 0..dyn_struct.field_len() {
        let field = dyn_struct.field_at(field_index).unwrap();
        debug.field(
            dyn_struct.name_at(field_index).unwrap(),
            &field as &dyn fmt::Debug,
        );
    }
    debug.finish()
}
