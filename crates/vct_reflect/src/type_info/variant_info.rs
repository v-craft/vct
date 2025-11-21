use core::{fmt, error};
use alloc::boxed::Box;
use vct_os::sync::Arc;
use vct_utils::collections::HashMap;
use crate::type_info::{
    CustomAttributes, NamedField, UnnamedField, 
    attributes::impl_custom_attributes_fn,
    docs_macro::impl_docs_fn,
};

/// 用于表示变体类型的枚举
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum VariantType {
    /// 结构体
    ///
    /// ```ignore
    /// enum MyEnum {
    ///   A {
    ///     foo: usize
    ///   }
    /// }
    /// ```
    Struct,
    /// 枚举
    ///
    /// ```ignore
    /// enum MyEnum {
    ///   A(usize)
    /// }
    /// ```
    Tuple,
    /// 单元
    ///
    /// ```ignore
    /// enum MyEnum {
    ///   A
    /// }
    /// ```
    Unit,
}

/// 存储枚举中的结构体项信息的容器
/// 
/// ```ignore
/// enum MyEnum {
///   A {
///     foo: usize
///   }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct StructVariantInfo {
    name: &'static str,
    fields: Box<[NamedField]>,
    field_names: Box<[&'static str]>,
    field_indices: HashMap<&'static str, usize>,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl StructVariantInfo {
    impl_docs_fn!(docs);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新容器
    pub fn new(name: &'static str, fields: &[NamedField]) -> Self {
        let field_indices = fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.name(), index))
            .collect();

        let field_names = fields.iter().map(NamedField::name).collect();

        Self {
            name,
            fields: fields.to_vec().into_boxed_slice(),
            field_names,
            field_indices,
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取自身的变体名
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// 获取字段名列表
    #[inline]
    pub fn field_names(&self) -> &[&'static str] {
        &self.field_names
    }

    /// 根据字段名获取字段信息
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

    /// 修改自定义属性
    #[inline]
    pub fn with_custom_attributes(self, custom_attributes: CustomAttributes) -> Self {
        Self {
            custom_attributes: Arc::new(custom_attributes),
            ..self
        }
    }
}


/// 存储枚举中的元组项信息的容器
/// 
/// ```ignore
/// enum MyEnum {
///   B(usize)
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TupleVariantInfo {
    name: &'static str,
    fields: Box<[UnnamedField]>,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl TupleVariantInfo {
    impl_docs_fn!(docs);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新对象
    #[inline]
    pub fn new(name: &'static str, fields: &[UnnamedField]) -> Self {
        Self {
            name,
            fields: fields.to_vec().into_boxed_slice(),
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取自身的变体名
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
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

/// 存储枚举中单元项信息的容器
/// 
/// ```ignore
/// enum MyEnum {
///   C
/// }
/// ```
#[derive(Clone, Debug)]
pub struct UnitVariantInfo {
    name: &'static str,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl UnitVariantInfo {
    impl_docs_fn!(docs);
    impl_custom_attributes_fn!(custom_attributes);

    /// 创建新容器
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取自身的变体名
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
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

/// 用于表示类型错误的枚举
#[derive(Debug)]
pub struct VariantTypeError {
    /// 预期类型
    expected: VariantType,
    /// 实际类型
    received: VariantType,
}

impl fmt::Display for VariantTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "variant type mismatch: expected {:?}, received {:?}", self.expected, self.received)
    }
}

impl error::Error for VariantTypeError {}


/// 存储编译时枚举变体项信息的容器
#[derive(Clone, Debug)]
pub enum VariantInfo {
    /// 参考 [`StructVariantInfo`]
    Struct(StructVariantInfo),

    /// 参考 [`TupleVariantInfo`]
    Tuple(TupleVariantInfo),

    /// 参考 [`UnitVariantInfo`]
    Unit(UnitVariantInfo),
}

macro_rules! impl_cast_fn {
    ($name:ident : $kind:ident => $info:ident) => {
        /// 类型转换
        #[inline]
        pub fn $name(&self) -> Result<&$info, VariantTypeError> {
            match self {
                Self::$kind(info) => Ok(info),
                _ => Err(VariantTypeError {
                    expected: VariantType::$kind,
                    received: self.variant_type(),
                }),
            }
        }
    };
}

impl VariantInfo {
    impl_cast_fn!(as_struct_variant: Struct => StructVariantInfo);
    impl_cast_fn!(as_tuple_variant: Tuple => TupleVariantInfo);
    impl_cast_fn!(as_unit_variant: Unit => UnitVariantInfo);

    impl_custom_attributes_fn!(self => match self {
        Self::Struct(info) => info.custom_attributes(),
        Self::Tuple(info) => info.custom_attributes(),
        Self::Unit(info) => info.custom_attributes(),
    });

    /// 获取变体名
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Struct(info) => info.name(),
            Self::Tuple(info) => info.name(),
            Self::Unit(info) => info.name(),
        }
    }

    /// 获取变体类型
    #[inline]
    pub fn variant_type(&self) -> VariantType {
        match self {
            Self::Struct(_) => VariantType::Struct,
            Self::Tuple(_) => VariantType::Tuple,
            Self::Unit(_) => VariantType::Unit,
        }
    }

    /// 读取文档
    #[cfg(feature = "reflect_docs")]
    #[inline]
    pub fn docs(&self) -> Option<&str> {
        match self {
            Self::Struct(info) => info.docs(),
            Self::Tuple(info) => info.docs(),
            Self::Unit(info) => info.docs(),
        }
    }
}

