use core::fmt;
use alloc::{
    string::ToString,
    vec::Vec,
    boxed::Box,
    borrow::Cow,
};
use vct_utils::collections::HashMap;
use crate::{
    PartialReflect, Reflect, info::{
        MaybeTyped, ReflectKind, StructInfo, TypeInfo, TypePath
    }, ops::{
        ApplyError, ReflectMut, ReflectOwned, ReflectRef
    }
};

/// 一个可以运行时动态调整字段的容器
#[derive(Default)]
pub struct DynamicStruct {
    target_type: Option<&'static TypeInfo>,
    fields: Vec<Box<dyn PartialReflect>>,
    field_names: Vec<Cow<'static, str>>,
    field_indices: HashMap<Cow<'static, str>, usize>,
}

impl TypePath for DynamicStruct {
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicStruct"
    }

    fn short_type_path() -> &'static str {
        "DynamicStruct"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynamicStruct")
    }
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicStruct {
    /// 修改代表的底层类型
    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::Struct(_)),
                "expected TypeInfo::Struct but received: {target_type:?}"
            );
        }
        self.target_type = target_type;
    }

    /// 插入字段
    pub fn insert_boxed(
        &mut self,
        name: impl Into<Cow<'static, str>>,
        value: Box<dyn PartialReflect>,
    ) {
        let name: Cow<'static, str> = name.into();
        if let Some(index) = self.field_indices.get(&name) {
            self.fields[*index] = value;
        } else {
            self.fields.push(value);
            self.field_indices.insert(name.clone(), self.fields.len() - 1);
            self.field_names.push(name);
        }
    }

    /// 插入字段
    #[inline]
    pub fn insert<'a, T: PartialReflect>(&mut self, name: impl Into<Cow<'static, str>>, value: T) {
        self.insert_boxed(name, Box::new(value));
    }

    /// 获取字段序号
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.field_indices.get(name).copied()
    }
}

impl PartialReflect for DynamicStruct {
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
        let other = value.reflect_ref().as_struct()?;

        for (idx, other_field) in other.iter_fields().enumerate() {
            let name = other.name_at(idx).unwrap();
            if let Some(field) = self.field_mut(name) {
                field.try_apply(other_field)?;
            }
        }
        Ok(())
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

    
    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        // 手动内联
        let ReflectRef::Struct(other) = other.reflect_ref() else {
            return Some(false);
        };
        if self.field_len() != other.field_len() {
            return Some(false);
        }
        for (idx, other_field) in other.iter_fields().enumerate() {
            let name = other.name_at(idx).unwrap();
            if let Some(self_field) = self.field(name) {
                let result = self_field.reflect_partial_eq(other_field);
                if result != Some(true) {
                    return result;
                }
            } else {
                return Some(false);
            }
        } 
        Some(true)
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicStruct(")?;
        struct_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicStruct {}

impl fmt::Debug for DynamicStruct {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl<'a, N: Into<Cow<'static, str>>> FromIterator<(N, Box<dyn PartialReflect>)> for DynamicStruct {
    fn from_iter<T: IntoIterator<Item = (N, Box<dyn PartialReflect>)>>(fields: T) -> Self {
        let mut dynamic_struct = Self::default();
        for (name, value) in fields.into_iter() {
            dynamic_struct.insert_boxed(name, value);
        }
        dynamic_struct
    }
}

impl IntoIterator for DynamicStruct {
    type Item = Box<dyn PartialReflect>;
    type IntoIter = alloc::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicStruct {
    type Item = &'a dyn PartialReflect;
    type IntoIter = StructFieldIter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter_fields()
    }
}

pub trait Struct: PartialReflect {
    /// 获取字段的引用
    fn field(&self, name: &str) -> Option<&dyn PartialReflect>;

    /// 获取字段的可变引用
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect>;

    /// 获取字段的引用
    fn field_at(&self, index: usize) -> Option<&dyn PartialReflect>;

    /// 获取字段的可变引用
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    /// 获取字段名
    fn name_at(&self, index: usize) -> Option<&str>;

    /// 获取字段数
    fn field_len(&self) -> usize;

    /// 获取字段迭代器
    fn iter_fields(&self) -> StructFieldIter<'_>;

    /// 获取动态结构体
    fn to_dynamic_struct(&self) -> DynamicStruct {
        let mut dynamic_struct = DynamicStruct::default();
        dynamic_struct.set_target_type_info(self.get_target_type_info());
        for (i, val) in self.iter_fields().enumerate() {
            dynamic_struct.insert_boxed(self.name_at(i).unwrap().to_string(), val.to_dynamic());
        }
        dynamic_struct
    }

    /// 获取底层类型
    fn get_target_struct_info(&self) -> Option<&'static StructInfo> {
        self.get_target_type_info()?.as_struct().ok()
    }
}

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
    type Item = &'a dyn PartialReflect;

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
    fn field(&self, name: &str) -> Option<&dyn PartialReflect> {
        self.field_indices.get(name).map(|index| &*self.fields[*index])
    }

    #[inline]
    fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect> {
        self.field_indices.get(name).map(|index| &mut *self.fields[*index])
    }

    #[inline]
    fn field_at(&self, index: usize) -> Option<&dyn PartialReflect> {
        self.fields.get(index).map(|value| &**value)
    }

    #[inline]
    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
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
            target_type: self.get_target_type_info(),
            fields: self.fields.iter().map(|val| val.to_dynamic()).collect(),
            field_names: self.field_names.clone(),
            field_indices: self.field_indices.clone(),
        }
    }
}

pub trait GetStructField {
    fn get_field<T: Reflect>(&self, name: &str) -> Option<&T>;
    fn get_field_mut<T: Reflect>(&mut self, name: &str) -> Option<&mut T>;
}

impl<S: Struct> GetStructField for S {
    fn get_field<T: Reflect>(&self, name: &str) -> Option<&T> {
        self.field(name)
            .and_then(|value| value.try_downcast_ref::<T>())
    }

    fn get_field_mut<T: Reflect>(&mut self, name: &str) -> Option<&mut T> {
        self.field_mut(name)
            .and_then(|value| value.try_downcast_mut::<T>())
    }
}

impl GetStructField for dyn Struct {
    #[inline]
    fn get_field<T: Reflect>(&self, name: &str) -> Option<&T> {
        self.field(name)
            .and_then(|value| value.try_downcast_ref::<T>())
    }

    #[inline]
    fn get_field_mut<T: Reflect>(&mut self, name: &str) -> Option<&mut T> {
        self.field_mut(name)
            .and_then(|value| value.try_downcast_mut::<T>())
    }
}

pub fn struct_partial_eq<S: Struct + ?Sized>(
    x: &S,
    y: &dyn PartialReflect
) -> Option<bool> {
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

pub fn struct_debug(dyn_struct: &dyn Struct, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_struct(
        dyn_struct
            .get_target_type_info()
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
