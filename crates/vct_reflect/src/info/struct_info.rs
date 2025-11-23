use alloc::boxed::Box;
use vct_utils::collections::HashMap;
use vct_os::sync::Arc;
use crate::{
    ops::Struct,
    info::{
        CustomAttributes, Generics, NamedField, 
        Type, TypePath, 
        docs_macro::impl_docs_fn,
        attributes::impl_custom_attributes_fn, 
        generics::impl_generic_fn, 
        type_struct::impl_type_fn
    }
};

/// 存储编译时结构体信息的容器
#[derive(Clone, Debug)]
pub struct StructInfo {
    ty: Type,
    generics: Generics,
    fields: Box<[NamedField]>,
    field_names: Box<[&'static str]>,
    field_indices: HashMap<&'static str, usize>,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl StructInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新容器
    /// 
    /// - fields 在容器内部的顺序是固定的
    pub fn new<T: TypePath + Struct>(fields: &[NamedField]) -> Self {
        let field_indices = fields
            .iter()
            .enumerate()
            .map(|(index,  field)| (field.name(), index))
            .collect();

        let field_names = fields.iter().map(NamedField::name).collect();

        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            fields: fields.to_vec().into_boxed_slice(),
            field_names,
            field_indices,
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取字段名列表的切片
    #[inline]
    pub fn field_names(&self) -> &[&'static str] {
        &self.field_names
    }

    /// 根据字段名查询字段详情
    #[inline]
    pub fn field(&self, name: &str) -> Option<&NamedField> {
        self.field_indices
            .get(name)
            .map(|index| &self.fields[*index])
    }

    /// 根据索引（序号）查询字段详情
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&NamedField> {
        self.fields.get(index)
    }

    /// 查询字段的索引（序号）
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.field_indices.get(name).copied()
    }

    /// 获取字段的迭代器
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, NamedField> {
        self.fields.iter()
    }

    /// 获取字段总数
    #[inline]
    pub fn field_len(&self) -> usize {
        self.fields.len()
    }

    /// 修改属性（覆盖，而非添加）
    #[inline]
    pub fn with_custom_attributes(self, custom_attributes: CustomAttributes) -> Self {
        Self {
            custom_attributes: Arc::new(custom_attributes),
            ..self
        }
    }
}
