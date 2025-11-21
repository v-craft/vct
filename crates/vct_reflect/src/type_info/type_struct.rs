use core::{
    any::{Any, TypeId},
    fmt::{Debug, Formatter},
    hash::Hash,
};

use crate::{
    TypePath, TypePathTable
};

/// 一个用于表示 Rust 类型的结构体
#[derive(Copy, Clone)]
pub struct Type {
    type_path_table: TypePathTable,
    type_id: TypeId,
}

impl Type {
    /// 基于指定实现了 [`TypePath`] 的类型创建新对象
    #[inline]
    pub fn of<T: TypePath + ?Sized>() -> Self {
        Self {
            type_path_table: TypePathTable::of::<T>(),
            type_id: TypeId::of::<T>(),
        }
    }

    /// 返回类型的 [`TypeId`]
    #[inline(always)]
    pub fn id(&self) -> TypeId {
        self.type_id
    }

    /// 参考 [`TypePath::type_path`]
    #[inline]
    pub fn path(&self) -> &'static str {
        self.type_path_table.path()
    }

    /// 参考 [`TypePath::short_type_path`]
    #[inline]
    pub fn short_path(&self) -> &'static str {
        self.type_path_table.short_path()
    }

    /// 参考 [`TypePath::type_ident`]
    #[inline]
    pub fn ident(&self) -> Option<&'static str> {
        self.type_path_table.ident()
    }

    /// 参考 [`TypePath::crate_name`]
    #[inline]
    pub fn crate_name(&self) -> Option<&'static str> {
        self.type_path_table.crate_name()
    }

    /// 参考 [`TypePath::module_path`]
    #[inline]
    pub fn module_path(&self) -> Option<&'static str> {
        self.type_path_table.module_path()
    }

    /// 返回 [`TypePathTable`] 
    #[inline]
    pub fn type_path_table(&self) -> &TypePathTable {
        &self.type_path_table
    }

    /// 检查类型是否匹配，仅比较 [`TypeId`]
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

/// 此实现基于 [`TypeId`]
impl PartialEq for Type {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for Type {}

/// 此实现基于 [`TypeId`]
impl Hash for Type {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

/// 此实现基于 [`TypePathTable::path()`]
impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.type_path_table.path())
    }
}

// 用在类型的 impl 块中，要求类型的某个字段是 `Type`
macro_rules! impl_type_fn {
    ($field:ident) => {
        $crate::type_info::type_struct::impl_type_fn!(self => &self.$field);
    };
    ($self:ident => $expr:expr) => {
        /// 获取底层类型的 [`Type`] 表示, expr 尽量简单
        #[inline]
        pub fn ty($self: &Self) -> &$crate::Type {
            $expr
        }

        /// 获取 [`TypeId`]
        #[inline]
        pub fn type_id(&self) -> ::core::any::TypeId {
            self.ty().id()
        }

        /// 获取类型路径
        #[inline]
        pub fn type_path(&self) -> &'static str {
            self.ty().path()
        }

        /// 获取 [`TypePathTable`]
        #[inline]
        pub fn type_path_table(&self) -> &$crate::TypePathTable {
            &self.ty().type_path_table()
        }

        /// 检查类型是否相同
        #[inline]
        pub fn is<T: ::core::any::Any>(&self) -> bool {
            self.ty().is::<T>()
        }
    };
}

pub(crate) use impl_type_fn;

