use alloc::{
    vec::Vec,
    boxed::Box,
    borrow::Cow,
};
use vct_utils::collections::HashMap;
use crate::{
    type_data::PartialReflect,
    type_info::{
        TypeInfo
    },
};

/// 一个可以运行时动态调整字段的容器
#[derive(Default)]
pub struct DynamicStruct {
    represented_type: Option<&'static TypeInfo>,
    fields: Vec<Box<dyn PartialReflect>>,
    field_names: Vec<Cow<'static, str>>,
    field_indices: HashMap<Cow<'static, str>, usize>,
}

impl DynamicStruct {
    /// 修改代表的底层类型
    pub fn set_represented_type(&mut self, represented_type: Option<&'static TypeInfo>) {
        if let Some(represented_type) = represented_type {
            assert!(
                matches!(represented_type, TypeInfo::Struct(_)),
                "expected TypeInfo::Struct but received: {represented_type:?}"
            );
        }
        self.represented_type = represented_type;
    }

    /// 插入字段
    pub fn insert_boxed<'a>(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: Box<dyn PartialReflect>,
    ) {
        let name: Cow<str> = name.into();
        if let Some(index) = self.field_indices.get(&name) {
            self.fields[*index] = value;
        } else {
            self.fields.push(value);
            self.field_indices.insert(Cow::Owned(name.clone().into_owned()), self.fields.len() - 1);
            self.field_names.push(Cow::Owned(name.into_owned()));
        }
    }

    /// 插入字段
    #[inline]
    pub fn insert<'a, T: PartialReflect>(&mut self, name: impl Into<Cow<'a, str>>, value: T) {
        self.insert_boxed(name, Box::new(value));
    }

    /// 获取字段序号
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.field_indices.get(name).copied()
    }
}

// /// 字段迭代器
// pub struct FieldIter<'a> {
//     struct_val: &'a dyn Struct,
//     index: usize,
// }

// impl<'a> FieldIter<'a> {
//     pub fn new(value: &'a dyn Struct) -> Self {
//         FieldIter {
//             struct_val: value,
//             index: 0,
//         }
//     }
// }

// impl<'a> Iterator for FieldIter<'a> {
//     type Item = &'a dyn PartialReflect;

//     fn next(&mut self) -> Option<Self::Item> {
//         let value = self.struct_val.field_at(self.index);
//         self.index += value.is_some() as usize;
//         value
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let size = self.struct_val.field_len();
//         (size, Some(size))
//     }
// }

// impl<'a> ExactSizeIterator for FieldIter<'a> {}

// pub trait Struct: PartialReflect {
//     /// 获取字段的引用
//     fn field(&self, name: &str) -> Option<&dyn PartialReflect>;
//     /// 获取字段的可变引用
//     fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect>;
//     /// 获取字段的引用
//     fn field_at(&self, index: usize) -> Option<&dyn PartialReflect>;
//     /// 获取字段的可变引用
//     fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;
//     /// 获取字段名
//     fn name_at(&self, index: usize) -> Option<&str>;
//     /// 获取字段数
//     fn field_len(&self) -> usize;
//     /// 获取底层类型
//     fn get_represented_struct_info(&self) -> Option<&'static StructInfo> {
//         self.get_represented_type_info()?.as_struct().ok()
//     }
//     /// 获取动态结构体
//     fn to_dynamic_struct(&self) -> DynamicStruct {
//         let mut dynamic_struct = DynamicStruct::default();
//         dynamic_struct.set_represented_type(self.get_represented_type_info());
//         for (i, value) in self.iter_fields().enumerate() {
//             dynamic_struct.insert_boxed(self.name_at(i).unwrap(), value.to_dynamic());
//         }
//         dynamic_struct
//     }
// }
