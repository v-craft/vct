
use alloc::boxed::Box;
use vct_os::sync::Arc;
use crate::{
    info::{
        CustomAttributes, Generics, Type, TypePath, UnnamedField, attributes::impl_custom_attributes_fn, docs_macro::impl_docs_fn, generics::impl_generic_fn, type_struct::impl_type_fn
    }, ops::TupleStruct
};

/// 存储编译时元组结构体信息的容器
#[derive(Clone, Debug)]
pub struct TupleStructInfo {
    ty: Type,
    generics: Generics,
    fields: Box<[UnnamedField]>,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl TupleStructInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);

    #[inline]
    pub fn new<T: TypePath + TupleStruct>(fields: &[UnnamedField]) -> Self {
        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            fields: fields.to_vec().into_boxed_slice(),
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 根据索引获取字段详情
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&UnnamedField> {
        self.fields.get(index)
    }

    /// 获取字段的迭代器
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, UnnamedField> {
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

