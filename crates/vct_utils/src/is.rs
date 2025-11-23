use core::any::{Any, TypeId};

/// 检查当前类型是否是指定类型 `T`，内部使用 [`TypeId`] 比较
pub trait Is {
    /// 检查当前类型是否是指定类型 `T`，内部使用 [`TypeId`] 比较
    /// 
    /// # 例
    /// 
    /// ```
    /// # use vct_reflect::Is;
    /// # use core::any::Any;
    /// 
    /// assert!(u32::is::<u32>());
    /// assert!(!usize::is::<u32>());
    /// ```
    fn is<T: Any>() -> bool;
}

impl <A: Any> Is for A {
    #[inline(always)]
    fn is<T: Any>() -> bool {
        TypeId::of::<A>() == TypeId::of::<T>()
    }
}
