use core::ops::Deref;
use alloc::{borrow::Cow, boxed::Box};
use vct_os::sync::Arc;

use crate::{
    Reflect, 
    info::{
        Type, TypePath,
        type_struct::impl_type_fn,
    },
};

/// 存储泛型类型参数信息的结构体
#[derive(Clone, Debug)]
pub struct TypeParamInfo {
    name: Cow<'static, str>,
    ty: Type,
    default: Option<Type>,
}

impl TypeParamInfo {
    // `is` `ty` `type_id` `type_path` `type_path_table`
    impl_type_fn!(ty);

    /// 基于指定类型和名称创建新对象
    #[inline]
    pub fn new<T: TypePath + ?Sized>(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            ty: Type::of::<T>(),
            default: None,
        }
    }

    /// 获取参数名
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// 获取参数默认类型
    #[inline]
    pub fn default(&self) -> Option<&Type> {
        self.default.as_ref()
    }

    /// 修改默认类型
    pub fn with_default<T: TypePath + ?Sized>(mut self) -> Self {
        self.default = Some(Type::of::<T>());
        self
    }
}

/// 存储泛型常量参数信息的结构体
#[derive(Clone, Debug)]
pub struct ConstParamInfo {
    name: Cow<'static, str>,
    ty: Type,
    // 常量参数是值且类型确定，因此使用 Arc<dyn Reflect> 存储
    default: Option<Arc<dyn Reflect>>,
}

impl ConstParamInfo {
    // `is` `ty` `type_id` `type_path` `type_path_table`
    impl_type_fn!(ty);

    /// 基于指定类型和名称创建新对象
    #[inline]
    pub fn new<T: TypePath + ?Sized>(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            ty: Type::of::<T>(),
            default: None,
        }
    }

    /// 获取参数名
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// 获取默认值
    #[inline]
    pub fn default(&self) -> Option<&dyn Reflect> {
        self.default.as_deref()
    }

    /// 设置默认值
    pub fn with_default<T: Reflect + 'static>(mut self, default: T) -> Self {
        let arc = Arc::new(default);

        #[cfg(not(feature = "std"))]
        #[expect(unsafe_code, reason = "unsized coercion is unstable without using std Arc")]
        // See: https://doc.rust-lang.org/alloc/sync/struct.Arc.html -- impl CoerceUnsized for Arc
        // TODO: Remove this after CoerceUnsized stabilization.
        let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const dyn Reflect) };

        self.default = Some(arc);
        self
    }
}

/// 表示单个泛型参数的枚举
#[derive(Clone, Debug)]
pub enum GenericInfo {
    /// 泛型的类型参数
    Type(TypeParamInfo),
    /// 泛型的常量参数
    Const(ConstParamInfo),
}

impl From<TypeParamInfo> for GenericInfo {
    #[inline]
    fn from(value: TypeParamInfo) -> Self {
        Self::Type(value)
    }
}

impl From<ConstParamInfo> for GenericInfo {
    #[inline]
    fn from(value: ConstParamInfo) -> Self {
        Self::Const(value)
    }
}

impl GenericInfo {
    // `is` `ty` `type_id` `type_path` `type_path_table`
    impl_type_fn!(self => match self {
        Self::Type(info) => info.ty(),
        Self::Const(info) => info.ty(),
    });

    /// 获取泛型名
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        match self {
            Self::Type(info) => info.name(),
            Self::Const(info) => info.name(),
        }
    }

    /// 判断是不是常量参数
    #[inline]
    pub fn is_const(&self) -> bool {
        match self {
            Self::Type(_) => false,
            Self::Const(_) => true,
        }
    }
}

/// 用于记录类型泛型参数列表的结构体
#[derive(Clone, Default, Debug)]
pub struct Generics(Box<[GenericInfo]>);

impl Generics {
    /// 创建空对象
    #[inline]
    pub fn new() -> Self {
        Self(Box::new([]))
    }

    /// 根据参数名查询泛型参数
    /// 
    /// 线性时间复杂度
    pub fn get(&self, name: &str) -> Option<&GenericInfo> {
        self.0.iter().find(|info| info.name() == name)
    }

    /// 末尾插入新参数
    /// 
    /// 线性时间复杂度
    pub fn with(mut self, info: impl Into<GenericInfo>) -> Self {
        self.0 = IntoIterator::into_iter(self.0)
            .chain(core::iter::once(info.into()))
            .collect();
        self
    }
}

impl<T: Into<GenericInfo>> FromIterator<T> for Generics {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl Deref for Generics {
    type Target = [GenericInfo];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

macro_rules! impl_generic_fn {
    ($field:ident) => {
        $crate::info::generics::impl_generic_fn!(self => &self.$field);

        /// 替换自身的泛型信息结构体
        pub fn with_generics(
            mut self, 
            generics: $crate::info::Generics
        ) -> Self {
            self.$field = generics;
            self
        }
    };
    ($self:ident => $expr:expr) => {
        /// 根据表达式获取 self 中的泛型信息结构体
        pub fn generics($self: &Self) -> &$crate::info::Generics {
            $expr
        }
    };
}

pub(crate) use impl_generic_fn;
