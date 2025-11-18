#![expect(unsafe_code, reason = "SyncCell requires unsafe code.")]

//! 不稳定 API [`std::sync::Exclusive`] 的一种实现
//!
//! [`std::sync::Exclusive`]: https://doc.rust-lang.org/nightly/std/sync/struct.Exclusive.html

use core::ptr;

/// 参考 [`Exclusive`](https://doc.rust-lang.org/nightly/std/sync/struct.Exclusive.html)
#[repr(transparent)]
pub struct SyncCell<T: ?Sized> {
    inner: T,
}

/// `Sync` 运行多线程访问不可变引用
///
/// 对应 `Sync` 的 T 类型，提供 `as_ref` 函数获取不可变引用。
/// 队伍 `!Sync` 的 T 类型，禁止 `as_ref` 函数，从而在自身 `Sync` 的同时保证安全性。
unsafe impl<T: ?Sized> Sync for SyncCell<T> {}

impl<T: Sized> SyncCell<T> {
    /// 从给定值构建新的 `SyncCell` 实例
    #[inline]
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }

    /// 析构自身，并移动出内部值
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: ?Sized> SyncCell<T> {
    /// 获取可变引用
    ///
    /// 使用时需遵守别名规则
    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// 只有内部 T 是 Sync 时才提供此函数，保证安全性
    #[inline]
    pub const fn as_ref(&self) -> &T
    where
        T: Sync,
    {
        &self.inner
    }

    /// 从可变引用获取 `SyncCell` 的可变引用
    #[inline]
    pub const fn from_mut(r: &mut T) -> &mut SyncCell<T> {
        let ptr = ptr::from_mut(r) as *mut SyncCell<T>;
        unsafe { &mut *ptr }
    }
}
