use alloc::boxed::Box;
use crate::{
    Reflect, 
    info::{
        Generics, UnnamedField, 
        Type, TypePath, 
        docs_macro::impl_docs_fn,
        generics::impl_generic_fn, 
        type_struct::impl_type_fn
    }
};

/// 存储编译时元组信息的容器
#[derive(Clone, Debug)]
pub struct TupleInfo {
    ty: Type,
    generics: Generics,
    fields: Box<[UnnamedField]>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl TupleInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    /// 创建新容器
    /// 
    /// - 内部字段顺序是固定的
    #[inline]
    pub fn new<T: Reflect + TypePath>(fields: &[UnnamedField]) -> Self {
        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            fields: fields.to_vec().into_boxed_slice(),
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
}



