use alloc::{
    boxed::Box, format, string::String
};
use vct_os::sync::Arc;
use vct_utils::collections::HashMap;
use crate::type_info::{
    CustomAttributes, Generics, Type,
    VariantInfo, TypePath,
    attributes::impl_custom_attributes_fn, 
    docs_macro::impl_docs_fn, 
    generics::impl_generic_fn, 
    type_struct::impl_type_fn,
};


#[derive(Clone, Debug)]
pub struct EnumInfo {
    ty: Type,
    generics: Generics,
    variants: Box<[VariantInfo]>,
    variant_names: Box<[&'static str]>,
    variant_indices: HashMap<&'static str, usize>,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl EnumInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新容器
    pub fn new<TEnum: TypePath /*+Enum*/>(variants: &[VariantInfo]) -> Self {
        let variant_indices = variants
            .iter()
            .enumerate()
            .map(|(index, variant)| (variant.name(), index))
            .collect();

        let variant_names = variants.iter().map(VariantInfo::name).collect();

        Self {
            ty: Type::of::<TEnum>(),
            generics: Generics::new(),
            variants: variants.to_vec().into_boxed_slice(),
            variant_names,
            variant_indices,
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 修改属性（覆盖，而非添加）
    #[inline]
    pub fn with_custom_attributes(self, custom_attributes: CustomAttributes) -> Self {
        Self {
            custom_attributes: Arc::new(custom_attributes),
            ..self
        }
    }

    /// 获取变体名列表
    #[inline]
    pub fn variant_names(&self) -> &[&'static str] {
        &self.variant_names
    }

    /// 根据变体名获取变体信息
    #[inline]
    pub fn variant(&self, name: &str) -> Option<&VariantInfo> {
        self.variant_indices
            .get(name)
            .map(|index| &self.variants[*index])
    }

    /// 根据索引获取变体信息
    #[inline]
    pub fn variant_at(&self, index: usize) -> Option<&VariantInfo> {
        self.variants.get(index)
    }

    /// 获取变体的索引
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.variant_indices.get(name).copied()
    }

    /// 返回变体的完整路径表示
    #[inline]
    pub fn variant_path(&self, name: &str) -> String {
        format!("{}::{name}", self.type_path())
    }

    /// 检查给定名称的变体是否存在
    #[inline]
    pub fn contains_variant(&self, name: &str) -> bool {
        self.variant_indices.contains_key(name)
    }

    /// 获取变体的迭代器
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, VariantInfo> {
        self.variants.iter()
    }

    /// 获取变体的数量
    #[inline]
    pub fn variant_len(&self) -> usize {
        self.variants.len()
    }
}
