use core::any::{Any, TypeId};
use alloc::{
    boxed::Box,
    string::String,
};
use crate::info::TypeInfo;
use vct_os::sync::{
    OnceLock, RwLock, PoisonError,
};
use vct_utils::TypeIdMap;


mod sealed {
    use super::TypeInfo;
    pub trait TypedProperty: 'static {}

    impl TypedProperty for alloc::string::String {}
    impl TypedProperty for TypeInfo {}
}

use sealed::TypedProperty;

pub struct NonGenericTypeCell<T: TypedProperty>(OnceLock<T>);
pub type NonGenericTypeInfoCell = NonGenericTypeCell<TypeInfo>;
pub type NonGenericTypePathCell = NonGenericTypeCell<String>;

impl<T: TypedProperty> NonGenericTypeCell<T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    #[inline]
    pub fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.0.get_or_init(f)
    }
}

impl<T: TypedProperty> Default for NonGenericTypeCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// 应仅作为 static 变量使用
pub struct GenericTypeCell<T: TypedProperty>(RwLock<TypeIdMap<&'static T>>);
pub type GenericTypeInfoCell = GenericTypeCell<TypeInfo>;
pub type GenericTypePathCell = GenericTypeCell<String>;

impl<T: TypedProperty> GenericTypeCell<T> {
    pub const fn new() -> Self {
        Self(RwLock::new(TypeIdMap::new()))
    }

    #[inline]
    pub fn get_or_insert<G, F>(&self, f: F) -> &T
    where
        G: Any + ?Sized,
        F: FnOnce() -> T,
    {
        // 分离以削减代码编译次数
        self.get_or_insert_by_type_id(TypeId::of::<G>(), f)
    }

    // 分离以削减代码编译次数
    fn get_or_insert_by_type_id<F>(&self, type_id: TypeId, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        match self.get_by_type_id(type_id) {
            Some(info) => info,
            None => self.insert_by_type_id(type_id, f()),
        }
    }

    // 分离以削减代码编译次数
    fn get_by_type_id(&self, type_id: TypeId) -> Option<&T> {
        self.0
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&type_id)
            .copied()
    }

    // 分离以削减代码编译次数
    fn insert_by_type_id(&self, type_id: TypeId, value: T) -> &T {
        let mut write_lock = self.0
            .write()
            .unwrap_or_else(PoisonError::into_inner);

        write_lock
            .entry(type_id)
            .insert({
                // 通过 leak 获取静态生命周期的引用。
                // GenericTypeCell 应当仅作为 static 变量使用，此时插入的数据本身就不会被释放，
                // 因此将值 leak 没有负面作用。
                Box::leak(Box::new(value))
            })
            .get()
    }
}

impl<T: TypedProperty> Default for GenericTypeCell<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

