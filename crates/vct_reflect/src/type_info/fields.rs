
use core::fmt::Display;
use alloc::borrow::Cow;

use vct_os::sync::Arc;

use crate::{CustomAttributes, MaybeTyped, Type, TypeInfo, TypePath, type_info::{attributes::impl_custom_attributes_fn, docs_macro::impl_docs_fn, type_struct::impl_type_fn}};

/// 命名字段，如结构体的字段
#[derive(Clone, Debug)]
pub struct NamedField {
    name: &'static str,
    type_info: fn() -> Option<&'static TypeInfo>,
    ty: Type,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl NamedField {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新对象
    #[inline]
    pub fn new<T: MaybeTyped + TypePath>(name: &'static str) -> Self {
        Self {
            name,
            type_info: T::maybe_type_info,
            ty: Type::of::<T>(),
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取字段名
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// 获取类型信息
    #[inline]
    pub fn type_info(&self) -> Option<&'static TypeInfo> {
        (self.type_info)()
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

/// 无名字段，如元组结构体
#[derive(Clone, Debug)]
pub struct UnnamedField {
    index: usize,
    type_info: fn() -> Option<&'static TypeInfo>,
    ty: Type,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl UnnamedField {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新对象
    #[inline]
    pub fn new<T: MaybeTyped + TypePath>(index: usize) -> Self {
        Self {
            index,
            type_info: T::maybe_type_info,
            ty: Type::of::<T>(),
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取字段名
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    /// 获取类型信息
    #[inline]
    pub fn type_info(&self) -> Option<&'static TypeInfo> {
        (self.type_info)()
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

/// 一个用于表示字段名的容器
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldId {
    Named(Cow<'static, str>),
    Unnamed(usize),
}

impl Display for FieldId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Named(name) => Display::fmt(name, f),
            Self::Unnamed(name) => Display::fmt(name, f),
        }
    }
}
