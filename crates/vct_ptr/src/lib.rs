#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
#![expect(unsafe_code, reason = "Raw pointers are inherently unsafe.")]

#[cfg(test)]
#[macro_use]
extern crate alloc;

use core::{
    cell::UnsafeCell,
    fmt::{self, Debug, Formatter, Pointer},
    marker::PhantomData,
    mem::{self, ManuallyDrop, MaybeUninit},
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

/// 一个只读版本的 [`NonNull<T>`] 。
#[repr(transparent)]
pub struct ConstNonNull<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> ConstNonNull<T> {
    /// 如果 `ptr` 非空，创建一个新的 `ConstNonNull` 对象。
    ///
    /// # Examples
    ///
    /// ```
    /// use vct_ptr::ConstNonNull;
    ///
    /// let x = 0u32;
    /// let ptr = ConstNonNull::<u32>::new(&raw const x).expect("ptr is null!");
    ///
    /// if let Some(ptr) = ConstNonNull::<u32>::new(core::ptr::null()) {
    ///     unreachable!();
    /// }
    /// ```
    #[inline]
    pub const fn new(ptr: *const T) -> Option<Self> {
        // NonNull::new(ptr.cast_mut()).map(Self)
        // `map` 还不是稳定的 const fn
        match NonNull::new(ptr.cast_mut()) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    /// 创建一个新 `ConstNonNull`  对象。
    ///
    /// # Examples
    ///
    /// ```
    /// use vct_ptr::ConstNonNull;
    ///
    /// let x = 0u32;
    /// let ptr = unsafe { ConstNonNull::new_unchecked(&raw const x) };
    /// ```
    ///
    /// 错误用法：
    ///
    /// ```rust,no_run
    /// use vct_ptr::ConstNonNull;
    ///
    /// // 空指针是未定义行为 ⚠️
    /// let ptr = unsafe { ConstNonNull::<u32>::new_unchecked(core::ptr::null()) };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
        unsafe { Self(NonNull::new_unchecked(ptr.cast_mut())) }
    }

    /// 返回指针指向对象的不可变引用
    ///
    /// # 安全性
    ///
    /// 调用此方法时需要遵守以下规则:
    ///
    /// - 指针必须满足[对齐要求]。
    /// - 指针指向的对象已经正确初始化。
    /// - 遵守 Rust 的别名规则，同一时刻只能存在一个可变引用或任意个不可变引用。
    ///
    /// 即使未使用返回的引用，也应该遵守这些规则。
    ///
    /// # Examples
    ///
    /// ```
    /// use vct_ptr::ConstNonNull;
    ///
    /// let mut x = 0u32;
    /// let ptr = ConstNonNull::new(&raw mut x).expect("ptr is null!");
    ///
    /// let ref_x = unsafe { ptr.as_ref() };
    /// println!("{ref_x}");
    /// ```
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn as_ref<'a>(&self) -> &'a T {
        unsafe { self.0.as_ref() }
    }
}

impl<T: ?Sized> From<NonNull<T>> for ConstNonNull<T> {
    #[inline]
    fn from(value: NonNull<T>) -> Self {
        ConstNonNull(value)
    }
}

impl<'a, T: ?Sized> From<&'a T> for ConstNonNull<T> {
    #[inline]
    fn from(value: &'a T) -> Self {
        ConstNonNull(NonNull::from(value))
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for ConstNonNull<T> {
    #[inline]
    fn from(value: &'a mut T) -> Self {
        ConstNonNull(NonNull::from(value))
    }
}

/// 此类型用于 [`Ptr`]、 [`PtrMut`]、 [`OwningPtr`] 和 [`MovingPtr`] ，表示满足[对齐要求]。
///
/// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Debug, Copy, Clone)]
pub struct Aligned;

/// 此类型用于 [`Ptr`]、 [`PtrMut`]、 [`OwningPtr`] 和 [`MovingPtr`] ，表示不满足[对齐要求]。
///
/// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Debug, Copy, Clone)]
pub struct Unaligned;

mod seal_aligned {
    pub trait Sealed {}
    impl Sealed for super::Aligned {}
    impl Sealed for super::Unaligned {}
}

/// 此 trait 仅对 [`Aligned`] 和 [`Unaligned`] 实现。
pub trait IsAligned: seal_aligned::Sealed {
    unsafe fn read_ptr<T>(ptr: *const T) -> T;
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);
    unsafe fn drop_in_place<T>(ptr: *mut T);
}

impl IsAligned for Aligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        // 安全性:
        // - 调用者需保证 `ptr` 有效且可读
        // - 调用者需保证 `ptr` 指向有效的 `T` 类型对象
        // - 这是对齐类型，因此需要保证 `ptr` 满足 `T` 类型的对齐要求
        unsafe { ptr.read() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        // 安全性:
        // - 调用者需保证 `src` 有效且可读
        // - 调用者需保证 `dst` 有效且可写
        // - 调用者需保证 `src` 和 `dst` 在 `count` 大小的区域不发生重叠
        // - 调用者需保证 `src` 和 `dst` 是对齐的
        unsafe {
            ptr::copy_nonoverlapping(src, dst, count);
        }
    }

    #[inline]
    unsafe fn drop_in_place<T>(ptr: *mut T) {
        // 安全性:
        // - 调用者需保证 `ptr` 有效且可读可写
        // - 调用者需保证 `ptr` 指向有效的 `T` 类型实例
        // - 调用者需要保证 `ptr` 是有效且目标可 `drop` 的
        // - 调用者需要保证此函数执行后 `ptr` 不能再被使用
        // - 这是对齐类型，因此需要保证 `ptr` 满足 `T` 类型的对齐要求
        unsafe {
            ptr::drop_in_place(ptr);
        }
    }
}

impl IsAligned for Unaligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        // 安全性:
        // - 调用者需保证 `ptr` 有效且可读
        // - 调用者需保证 `ptr` 指向有效的 `T` 类型对象
        unsafe { ptr.read_unaligned() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        // 安全性:
        // - 调用者需保证 `src` 有效且可读
        // - 调用者需保证 `dst` 有效且可写
        // - 调用者需保证 `src` 和 `dst` 在 `count` 大小的区域不发生重叠
        // - 这将基于字节拷贝，因此始终满足对齐要求
        unsafe {
            ptr::copy_nonoverlapping::<u8>(
                src.cast::<u8>(),
                dst.cast::<u8>(),
                count * size_of::<T>(),
            );
        }
    }

    #[inline]
    unsafe fn drop_in_place<T>(ptr: *mut T) {
        // 安全性:
        // - 调用者需保证 `ptr` 有效且可读可写
        // - 调用者需保证 `ptr` 指向有效的 `T` 类型实例
        // - 调用者需要保证 `ptr` 是有效且目标可 `drop` 的
        // - 调用者需要保证此函数执行后 `ptr` 不能再被使用
        unsafe {
            drop(ptr.read_unaligned());
        }
    }
}

/// 一个类型擦除的指向不可变对象的指针。
///
/// 可以把它看做 `&'a dyn Any` ，但它没有元数据且可指向非 Rust 对印的类型。
///
/// 这个类型试图模仿借用，因此：
/// - 它是不可变的，因此指向的对象不应该在它还存在时发生变化。
/// - 它必须始终指向一个有效的值。
/// - 生命周期 `'a` 准确地表示此指针多久有效。
/// - 如果 `A` 是 [`Aligned`]，指针需要满足其指向类型的[对齐要求]。
///
/// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Ptr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a u8, A)>);

/// 一个类型擦除的指向可变对象的指针。
///
/// 可以把它看做 `&'a mut dyn Any` ，但它没有元数据且可指向非 Rust 对印的类型。
///
/// 这个类型试图模仿借用，因此：
/// - 指针是可变且互斥的，因此自身不支持复制。
/// - 它必须始终指向一个有效的值。
/// - 生命周期 `'a` 准确地表示此指针多久有效。
/// - 如果 `A` 是 [`Aligned`]，指针需要满足其指向类型的[对齐要求]。
///
/// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[repr(transparent)]
pub struct PtrMut<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// 一个类型擦除的类似 [`Box`] 的指针。
///
/// 这个指针不负责目标的内存释放，因此通常用于执行栈区数据或 `Vec` 等容器托管的数据。
///
/// 概念上它有数据的所有权，因此你应该负责调用数据的 `Drop::drop` 。
/// 可以把他想象成 `&'a mut ManuallyDrop<dyn Any>` 。
///
/// 这个类型试图模仿借用，因此：
/// - 指针是可变且互斥的，因此自身不支持复制。
/// - 它必须始终指向一个有效的值。
/// - 生命周期 `'a` 准确地表示此指针多久有效。
/// - 如果 `A` 是 [`Aligned`]，指针需要满足其指向类型的[对齐要求]。
///
/// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
#[repr(transparent)]
pub struct OwningPtr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// 一个类似 [`Box`] 的指针，用于廉价地“移动”大对象。
///
/// 这个指针不负责目标的内存释放，因此通常用于执行栈区数据或 `Vec` 等容器托管的数据。
///
/// 概念上它有数据的所有权且知晓其类型，因此会在自身消亡时执行目标的 `Drop::drop` 。
///
/// 这个类型试图模仿借用，因此：
/// - 指针是可变且互斥的，因此自身不支持复制。
/// - 它必须始终指向一个有效的值。
/// - 生命周期 `'a` 准确地表示此指针多久有效。
/// - 不支持指针的算术运算。
/// - 如果 `A` 是 [`Aligned`]，指针需要满足其指向类型的[对齐要求]。
///
/// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
#[repr(transparent)]
pub struct MovingPtr<'a, T, A: IsAligned = Aligned>(NonNull<T>, PhantomData<(&'a mut T, A)>);

trait DebugEnsureAligned {
    fn debug_ensure_aligned(self) -> Self;
}

// miri 运行时自带指针对齐检查
#[cfg(all(debug_assertions, not(miri)))]
impl<T: Sized> DebugEnsureAligned for *mut T {
    #[track_caller]
    fn debug_ensure_aligned(self) -> Self {
        assert!(
            self.is_aligned(),
            "pointer is not aligned. Address {:p} does not have alignment {} for type {}",
            self,
            align_of::<T>(),
            core::any::type_name::<T>()
        );
        self
    }
}

#[cfg(any(not(debug_assertions), miri))]
impl<T: Sized> DebugEnsureAligned for *mut T {
    #[inline(always)]
    fn debug_ensure_aligned(self) -> Self {
        self
    }
}

impl<'a, A: IsAligned> Ptr<'a, A> {
    /// 从裸指针创建新对象。
    ///
    /// # 安全性
    /// - `inner`必须指向一个有效的值.
    /// - 生命周期 `'a` 需要限制 [`Ptr`] 的有效性，在 [`Ptr`] 处于活动状态时，除了通过 [`UnsafeCell`] ，不能改变指向的目标。
    /// - 如果 `A` 是 [`Aligned`] ，那么 `inner` 必须满足指向类型的[对齐要求]。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// 获取底层指针并擦除生命周期
    ///
    /// # 安全性
    /// 如果目标不可变，不能通过 `*mut T` 修改目标数据。
    ///
    /// 如果可行，尽量使用 [`deref`](Self::deref) 而非此函数（前者保留了生命周期）。
    #[inline]
    pub const fn as_ptr(self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// 从此指针获取一个带生命周期的 `&T`
    ///
    /// # 安全性
    /// - `T` 必须是指针指向的正确类型
    /// - 如果 `A` 是 [`Unaligned`]，指针依然需要满足 `T` 类型的[对齐要求]。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn deref<T>(self) -> &'a T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { &*ptr }
    }

    /// 将 [`Ptr`] 变为 [`PtrMut`]
    ///
    /// # 安全性
    /// - `Ptr` 指向的数据必须是可写的。
    /// - 必须保证没有其他活跃的可变引用执行 `Ptr` 的目标数据。
    /// - 新的指向相同数据的 [`PtrMut`] 必须在旧者被丢弃后才能创建
    #[inline]
    pub const unsafe fn into_mut(self) -> PtrMut<'a, A> {
        PtrMut(self.0, PhantomData)
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ptr<'a> {
    #[inline]
    fn from(val: &'a T) -> Self {
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a, A: IsAligned> PtrMut<'a, A> {
    /// 从裸指针创建新对象。
    ///
    /// # 安全性
    /// - `inner`必须指向一个有效的值.
    /// - 读写 `inner` 时需要有正确的保证。
    /// - 如果 `A` 是 [`Aligned`] ，那么 `inner` 必须满足指向类型的[对齐要求]。
    /// - 生命周期 `'a` 需要限制 [`PtrMut`] 的有效性，在 [`PtrMut`] 处于活动状态时，不能通过其他方式读写指向的目标。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// 获取底层指针并擦除生命周期
    ///
    /// 如果可行，尽量使用 [`deref_mut`](Self::deref_mut) 而非此函数（前者保留了生命周期）。
    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// 从此指针获取一个带生命周期的 `&mut T`
    ///
    /// # 安全性
    /// - `T` 必须是指针指向的正确类型
    /// - 如果 `A` 是 [`Unaligned`]，指针依然需要满足 `T` 类型的[对齐要求]。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn deref_mut<T>(self) -> &'a mut T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        // 安全性: 调用者需要保证指针指向的 `T` 是可以正确获取引用的
        unsafe { &mut *ptr }
    }

    /// 获取不可变指针
    #[inline]
    pub const fn as_ref(&self) -> Ptr<'_, A> {
        // 安全性: `PtrMut` 的有效性保证应该是 `Ptr` 的超集，且 `Ptr` 有效期间不能使用 `PtrMut` 修改数据。
        unsafe { Ptr::new(self.0) }
    }

    /// 从当前的 [`PtrMut`] 获取一个更小的生命周期版本，以临时传递或局部使用
    #[inline]
    pub const fn reborrow(&mut self) -> PtrMut<'_, A> {
        // 安全性：在新的 `PtrMut` 有效期间，原 `PtrMut` 不能使用
        unsafe { PtrMut::new(self.0) }
    }

    /// 将 [`PtrMut`] 转换为 [`OwningPtr`]
    #[inline]
    pub const unsafe fn promote(self) -> OwningPtr<'a, A> {
        OwningPtr(self.0, PhantomData)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for PtrMut<'a> {
    #[inline]
    fn from(val: &'a mut T) -> Self {
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a> OwningPtr<'a> {
    /// 此函数用于削减编译耗时
    /// 此处代码被编译的次数与类型数 `T` 一致，内联 `make` 则将与类型数与函数数的乘积  `T` * `F` 一致
    unsafe fn make_internal<T>(temp: &mut ManuallyDrop<T>) -> OwningPtr<'_> {
        unsafe { PtrMut::from(temp).promote() }
    }

    /// 从目标创建一个 [`OwningPtr`] 对象并消耗其值，保证不会出现二次 drop 的情况
    #[inline]
    pub fn make<T, F: FnOnce(OwningPtr<'_>) -> R, R>(val: T, f: F) -> R {
        let mut val = ManuallyDrop::new(val);
        // 安全性：此处传入的 `OwningPtr` 指向函数内的 val，因此它必须在闭包中被消耗，不能保存到外部
        f(unsafe { Self::make_internal(&mut val) })
    }
}

impl<'a> OwningPtr<'a, Unaligned> {
    /// 消耗 [`OwningPtr`] 并获取底层的 `T` 数据。
    ///
    /// # 安全性
    /// - `T` 必须是 [`OwningPtr`] 指向的数据类型。
    #[inline]
    pub const unsafe fn read_unaligned<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>();
        // 安全性：调用者需自行保证 `read_unaligned` 的安全性
        unsafe { ptr.read_unaligned() }
    }
}

impl<'a, A: IsAligned> OwningPtr<'a, A> {
    /// 从裸指针创建新对象。
    ///
    /// # 安全性
    /// - `inner`必须指向一个有效的值.
    /// - 读写 `inner` 时需要有正确的保证。
    /// - 如果 `A` 是 [`Aligned`] ，那么 `inner` 必须满足指向类型的[对齐要求]。
    /// - 生命周期 `'a` 需要限制 [`OwningPtr`] 的有效性，在 [`OwningPtr`] 处于活动状态时，不能通过其他方式读写指向的目标。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// 获取底层指针并擦除生命周期
    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// 消耗 [`OwningPtr`] 并复制其指向的数据 `T` 。
    ///
    /// # 安全性
    /// - `T` 必须是指针指向的正确类型
    /// - 如果 `A` 是 [`Unaligned`]，指针依然需要满足 `T` 类型的[对齐要求]。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn read<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { ptr.read() }
    }

    /// 消耗 [`OwningPtr`] 并调用其底层数据 `T` 的 `drop`
    ///
    /// # 安全性
    /// - `T` 必须是指针指向的正确类型
    /// - 如果 `A` 是 [`Unaligned`]，指针依然需要满足 `T` 类型的[对齐要求]。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn drop_as<T>(self) {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe {
            ptr.drop_in_place();
        }
    }

    /// 从所有权指针获取不可变指针
    #[inline]
    pub const fn as_ref(&self) -> Ptr<'_, A> {
        // 安全性: `OwningPtr` 的有效性保证应该是 `Ptr` 的超集，且 `Ptr` 有效期间不能使用 `OwningPtr` 修改数据。
        unsafe { Ptr::new(self.0) }
    }

    /// 从所有权指针获取可变指针
    #[inline]
    pub const fn as_mut(&mut self) -> PtrMut<'_, A> {
        unsafe { PtrMut::new(self.0) }
    }

    /// 根据类型转换到 [`MovingPtr`].
    #[inline]
    pub const unsafe fn cast<T>(self) -> MovingPtr<'a, T, A> {
        MovingPtr(self.0.cast::<T>(), PhantomData)
    }
}

macro_rules! impl_ptr {
    ($ptr:ident) => {
        impl<'a> $ptr<'a, Aligned> {
            /// 去除指针的对齐保证
            #[inline]
            pub const fn to_unaligned(self) -> $ptr<'a, Unaligned> {
                $ptr(self.0, PhantomData)
            }
        }

        impl<'a, A: IsAligned> From<$ptr<'a, A>> for NonNull<u8> {
            #[inline]
            fn from(ptr: $ptr<'a, A>) -> Self {
                ptr.0
            }
        }

        impl<A: IsAligned> $ptr<'_, A> {
            /// 计算偏移后的指针。
            /// 因为类型擦除，偏移量将直接以字节为单位。
            #[inline]
            pub const unsafe fn byte_offset(self, count: isize) -> Self {
                Self(
                    unsafe { NonNull::new_unchecked(self.as_ptr().offset(count)) },
                    PhantomData,
                )
            }

            /// 计算偏移后的指针，只能正向偏移。
            /// 因为类型擦除，偏移量将直接以字节为单位。
            #[inline]
            pub const unsafe fn byte_add(self, count: usize) -> Self {
                Self(
                    unsafe { NonNull::new_unchecked(self.as_ptr().add(count)) },
                    PhantomData,
                )
            }
        }

        impl<A: IsAligned> Pointer for $ptr<'_, A> {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                Pointer::fmt(&self.0, f)
            }
        }

        impl Debug for $ptr<'_, Aligned> {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}<Aligned>({:?})", stringify!($ptr), self.0)
            }
        }

        impl Debug for $ptr<'_, Unaligned> {
            #[inline]
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                write!(f, "{}<Unaligned>({:?})", stringify!($ptr), self.0)
            }
        }
    };
}

impl_ptr!(Ptr);
impl_ptr!(PtrMut);
impl_ptr!(OwningPtr);

impl<'a, T> MovingPtr<'a, T, Aligned> {
    /// 移除指针的对齐需求
    #[inline]
    pub const fn to_unaligned(self) -> MovingPtr<'a, T, Unaligned> {
        let value = MovingPtr(self.0, PhantomData);
        mem::forget(self);
        value
    }

    /// 从给定值的类型 `T` 创建 [`MovingPtr`]
    ///
    /// 作为更安全的替代，建议使用 [`move_as_ptr`] 。
    ///
    /// # 安全性
    /// - `value` 必须存储已初始化的 `T` 类型对象
    /// - 当返回的 [`MovingPtr`] 被使用后，`value` 应当被认为是未初始化的对象，除非显式使用了 [`core::mem::forget`]。
    #[inline]
    pub unsafe fn from_value(value: &'a mut MaybeUninit<T>) -> Self {
        MovingPtr(NonNull::from(value).cast::<T>(), PhantomData)
    }
}

impl<'a, T, A: IsAligned> MovingPtr<'a, T, A> {
    /// 从裸指针创建新对象。
    ///
    /// 作为更安全的替代，建议使用 [`move_as_ptr`] 。
    ///
    /// # 安全性
    /// - `inner`必须指向一个有效的值.
    /// - 读写 `inner` 时需要有正确的保证。
    /// - 如果 `A` 是 [`Aligned`] ，那么 `inner` 必须满足指向类型的[对齐要求]。
    /// - 生命周期 `'a` 需要限制 [`MovingPtr`] 的有效性，在 [`MovingPtr`] 处于活动状态时，不能通过其他方式读写指向的目标。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<T>) -> Self {
        Self(inner, PhantomData)
    }

    /// 消耗自身的部分字段
    ///
    /// 自身被消耗，返回指向 [`MaybeUninit<T>`] 的指针。
    ///
    /// 虽然此函数是安全的，但需要注意返回的值可能是不完整对象。
    ///
    /// # Example
    ///
    /// ```
    /// use core::mem::{offset_of, MaybeUninit, forget};
    /// use vct_ptr::{MovingPtr, move_as_ptr};
    /// # struct FieldAType(usize);
    /// # struct FieldBType(usize);
    /// # struct FieldCType(usize);
    /// # fn insert<T>(_ptr: MovingPtr<'_, T>) {}
    ///
    /// struct Parent {
    ///   field_a: FieldAType,
    ///   field_b: FieldBType,
    ///   field_c: FieldCType,
    /// }
    ///
    /// # let parent = Parent {
    /// #   field_a: FieldAType(0),
    /// #   field_b: FieldBType(0),
    /// #   field_c: FieldCType(0),
    /// # };
    ///
    /// // 将 `parent` 使用 Maybeuninit<> 包裹（不再自动 drop）
    /// // 然后创建一个同名的 `MovingPtr` 指向它，并托管其 drop
    /// move_as_ptr!(parent);
    ///
    /// let (partial_parent, ()) = MovingPtr::partial_move(parent, |parent_ptr| unsafe {
    ///   vct_ptr::deconstruct_moving_ptr!({
    ///     let Parent { field_a, field_b, field_c } = parent_ptr;
    ///   });
    ///   
    ///   insert(field_a);
    ///   insert(field_b);
    ///   forget(field_c);
    /// });
    ///
    /// unsafe {
    ///   vct_ptr::deconstruct_moving_ptr!({
    ///     let MaybeUninit::<Parent> { field_a: _, field_b: _, field_c } = partial_parent;
    ///   });
    ///
    ///   insert(field_c);
    /// }
    /// ```
    ///
    /// [`forget`]: core::mem::forget
    #[inline]
    pub fn partial_move<R>(
        self,
        f: impl FnOnce(MovingPtr<'_, T, A>) -> R,
    ) -> (MovingPtr<'a, MaybeUninit<T>, A>, R) {
        let partial_ptr = self.0;
        let ret = f(self);
        (
            MovingPtr(partial_ptr.cast::<MaybeUninit<T>>(), PhantomData),
            ret,
        )
    }

    /// 读取指针指向的值
    #[inline]
    pub fn read(self) -> T {
        // 安全性:
        // - `self.0` 必须对读取有效，因为此类型拥有它指向的值.
        // - `self.0` 必须始终指向一个有效的 `T` 类型实例。
        // - 如果 `A` 是 [`Aligned`]，那么指针需要满足 `T` 类型的对齐要求
        let value = unsafe { A::read_ptr(self.0.as_ptr()) };
        mem::forget(self);
        value
    }

    /// 将此指针指向的值写入指定位置。
    ///
    /// 该操作不会销毁 `dst` 上已有的值；调用者需负责确保 `dst` 上原有值被正确处理（例如先调用 `drop`）。
    ///
    /// # 安全性
    /// - `dst` 必须指向可写且有效的内存。
    /// - 若类型参数 `A` 为 [`Aligned`]，则 `dst` 必须满足 `T` 的[对齐要求]。
    ///
    /// [对齐要求]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn write_to(self, dst: *mut T) {
        let src = self.0.as_ptr();
        mem::forget(self);
        // SAFETY:
        //  `src` 必须可读，因为该指针被视为拥有其指向的值。
        // - 调用者必须保证 `dst` 可写且有效。
        // - 当 `A` 为 `Aligned` 时，调用者必须保证 `dst` 满足 `T` 的对齐要求；
        //   同时 `src` 的对齐性应自行保证，不在此处检查。
        unsafe { A::copy_nonoverlapping(src, dst, 1) };
    }

    /// 将此指针指向的值写入指定位置。
    ///
    /// 此操作将预先丢弃 `dst` 中的值。
    #[inline]
    pub fn assign_to(self, dst: &mut T) {
        // 安全性：
        // - `dst` 为可变借用，必须指向一个有效的 `T` 实例。
        // - `dst` 指向的值必须可以被安全地丢弃（drop）。
        // - `dst` 不得与其他访问产生别名（在此期间不存在其他活动引用）。
        unsafe {
            ptr::drop_in_place(dst);
        }
        // 安全性：
        // - `dst` 为可变借用，必须可写且指向有效内存。
        // - `dst` 必须满足 `T` 的对齐要求。
        unsafe {
            self.write_to(dst);
        }
    }

    /// 为 `self` 的某个字段创建一个 [`MovingPtr`]。
    ///
    /// 该函数专为析构式移动（deconstructive moves）设计。
    ///
    /// 字段的正确字节偏移量可通过 [`core::mem::offset_of`] 获取。
    ///
    /// # 安全性
    /// - `f` 必须返回指向 `T` 内部某个有效字段的非空指针。
    /// - 若 `A` 为 [`Aligned`]，则 `T` 不能是 `repr(packed)`。
    /// - 在此函数返回后，不应再把 `self` 当作完整值来访问或丢弃。尚未被移出的字段仍可单独访问或丢弃。
    /// - 此函数不得与其它对同一字段的访问别名（包括对同一字段的其它 `move_field` 调用），除非先对该字段调用了 [`forget`]。
    ///
    /// # Example
    ///
    /// ```
    /// use core::mem::offset_of;
    /// use vct_ptr::{MovingPtr, move_as_ptr};
    /// # struct FieldAType(usize);
    /// # struct FieldBType(usize);
    /// # struct FieldCType(usize);
    /// # fn insert<T>(_ptr: MovingPtr<'_, T>) {}
    ///
    /// struct Parent {
    ///   field_a: FieldAType,
    ///   field_b: FieldBType,
    ///   field_c: FieldCType,
    /// }
    ///
    /// let parent = Parent {
    ///    field_a: FieldAType(0),
    ///    field_b: FieldBType(0),
    ///    field_c: FieldCType(0),
    /// };
    ///
    /// // 将`parent`转换为`MovingPtr`
    /// move_as_ptr!(parent);
    ///
    /// unsafe {
    ///    let field_a = parent.move_field(|ptr| &raw mut (*ptr).field_a);
    ///    let field_b = parent.move_field(|ptr| &raw mut (*ptr).field_b);
    ///    let field_c = parent.move_field(|ptr| &raw mut (*ptr).field_c);
    ///    // 每次 insert 都可能 panic，在调用它们之前先保证 `parent_ptr` 不会被 drop 。
    ///    core::mem::forget(parent);
    ///    insert(field_a);
    ///    insert(field_b);
    ///    insert(field_c);
    /// }
    /// ```
    ///
    /// [`forget`]: core::mem::forget
    /// [`move_field`]: Self::move_field
    #[inline(always)]
    pub unsafe fn move_field<U>(&self, f: impl Fn(*mut T) -> *mut U) -> MovingPtr<'a, U, A> {
        MovingPtr(
            unsafe { NonNull::new_unchecked(f(self.0.as_ptr())) },
            PhantomData,
        )
    }
}

impl<'a, T, A: IsAligned> MovingPtr<'a, MaybeUninit<T>, A> {
    /// 为 `self` 的某个字段创建一个 [`MovingPtr`]。
    ///
    /// 本函数用于“析构式移动”（deconstructive moves）。
    ///
    /// 字段的正确字节偏移量可通过 [`core::mem::offset_of`] 获得。
    ///
    /// # 安全性
    /// - `f` 必须返回指向 `T` 内部某个有效字段的非空指针。
    /// - 若 `A` 为 [`Aligned`]，则 `T` 不能是 `repr(packed)`。
    /// - 在本函数返回后，不应再把 `self` 当作完整值来访问或丢弃。尚未被移出的字段仍可单独访问或丢弃。
    /// - 在持有字段指针期间，不能与对同一字段的其它访问发生别名（包括对同一字段的其它 `move_field` 调用），除非先对该字段调用了 [`core::mem::forget`]。
    ///
    /// [`forget`]: core::mem::forget
    /// [`move_field`]: Self::move_field
    #[inline(always)]
    pub unsafe fn move_maybe_uninit_field<U>(
        &self,
        f: impl Fn(*mut T) -> *mut U,
    ) -> MovingPtr<'a, MaybeUninit<U>, A> {
        let self_ptr = self.0.as_ptr().cast::<T>();
        // 安全性:
        // - 调用者必须保证 `U` 对应字段的类型正确且返回指针非空。
        // - `MaybeUninit<T>` 是 `repr(transparent)`，因此其内存布局应与 `T` 相同。
        let field_ptr = unsafe { NonNull::new_unchecked(f(self_ptr)) };
        MovingPtr(field_ptr.cast::<MaybeUninit<U>>(), PhantomData)
    }

    /// 将指向的 `MaybeUninit<T>` 视为已初始化并返回指向 `T` 的 `MovingPtr`。
    ///
    /// 参见：[`MaybeUninit::assume_init`]。
    ///
    /// # 安全性
    /// 调用者必须保证 `self` 指向的值已经完成初始化；否则会导致未定义行为。
    #[inline]
    pub unsafe fn assume_init(self) -> MovingPtr<'a, T, A> {
        let value = MovingPtr(self.0.cast::<T>(), PhantomData);
        mem::forget(self);
        value
    }
}

impl<T, A: IsAligned> Pointer for MovingPtr<'_, T, A> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&self.0, f)
    }
}

impl<T> Debug for MovingPtr<'_, T, Aligned> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MovingPtr<Aligned>({:?})", self.0)
    }
}

impl<T> Debug for MovingPtr<'_, T, Unaligned> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "MovingPtr<Unaligned>({:?})", self.0)
    }
}

impl<'a, T, A: IsAligned> From<MovingPtr<'a, T, A>> for OwningPtr<'a, A> {
    #[inline]
    fn from(value: MovingPtr<'a, T, A>) -> Self {
        // 安全性：
        // - `value.0` 必须始终指向类型 `T` 的有效值。
        // - 类型参数 `A` 在输入到输出间保持一致，保留相同的对齐保证。
        // - `value.0` 的来源（provenance）必须正确，以允许对 `T` 的读写。
        // - 生命周期 `'a` 在输入到输出间保持一致，保留相同的生命周期约束。
        // - `OwningPtr` 维持与 `MovingPtr` 相同的别名不变式。
        let ptr = unsafe { OwningPtr::new(value.0.cast::<u8>()) };
        mem::forget(value);
        ptr
    }
}

impl<'a, T> TryFrom<MovingPtr<'a, T, Unaligned>> for MovingPtr<'a, T, Aligned> {
    type Error = MovingPtr<'a, T, Unaligned>;
    #[inline]
    fn try_from(value: MovingPtr<'a, T, Unaligned>) -> Result<Self, Self::Error> {
        let ptr = value.0;
        if ptr.as_ptr().is_aligned() {
            mem::forget(value);
            Ok(MovingPtr(ptr, PhantomData))
        } else {
            Err(value)
        }
    }
}

impl<T> Deref for MovingPtr<'_, T, Aligned> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        let ptr = self.0.as_ptr().debug_ensure_aligned();
        unsafe { &*ptr }
    }
}

impl<T> DerefMut for MovingPtr<'_, T, Aligned> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.0.as_ptr().debug_ensure_aligned();
        unsafe { &mut *ptr }
    }
}

impl<T, A: IsAligned> Drop for MovingPtr<'_, T, A> {
    fn drop(&mut self) {
        unsafe { A::drop_in_place(self.0.as_ptr()) };
    }
}

/// 一个去除了长度信息的切片指针，以实现更好的性能
#[cfg(debug_assertions)]
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<&'a [T]>,
}

/// 一个去除了长度信息的切片指针，以实现更好的性能
#[cfg(not(debug_assertions))]
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> ThinSlicePtr<'a, T> {
    /// 索引切片但不进行边界检查
    ///
    /// # 安全性
    /// `index` 必须在边界内
    #[inline]
    pub const unsafe fn get(self, index: usize) -> &'a T {
        #[cfg(debug_assertions)]
        debug_assert!(index < self.len);

        let ptr = self.ptr.as_ptr();
        unsafe { &*ptr.add(index) }
    }
}

impl<'a, T> From<&'a [T]> for ThinSlicePtr<'a, T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        let ptr = slice.as_ptr().cast_mut();
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr.debug_ensure_aligned()) },
            #[cfg(debug_assertions)]
            len: slice.len(),
            _marker: PhantomData,
        }
    }
}

/// 根据偏移量创建一个悬垂指针
pub const fn dangling_with_align(align: NonZeroUsize) -> NonNull<u8> {
    debug_assert!(align.is_power_of_two(), "Alignment must be power of two.");

    unsafe { NonNull::new_unchecked(ptr::null_mut::<u8>().wrapping_add(align.get())) }
}

mod seal_unsafe_cell {
    use core::cell::UnsafeCell;
    pub trait Sealed {}
    impl<'a, T> Sealed for &'a UnsafeCell<T> {}
}

/// 为 [`UnsafeCell`] 添加一下额外的方法
pub trait UnsafeCellDeref<'a, T>: seal_unsafe_cell::Sealed {
    /// # 安全性
    /// - 返回的可变引用必须是唯一的，不得与对该 `UnsafeCell` 内容的任何其他可变或不可变引用别名。
    /// - 必须避免数据竞争：若多个线程可访问同一 `UnsafeCell`，所有写入必须有恰当的 happens-before 关系或使用原子操作。
    unsafe fn deref_mut(self) -> &'a mut T;

    /// # 安全性
    /// - 对返回值生命周期 `'a`，在此期间不得再构造指向 `UnsafeCell` 内容的可变引用。
    /// - 必须避免数据竞争：若多个线程可访问同一 `UnsafeCell`，所有写入必须有恰当的 happens-before 关系或使用原子操作。
    unsafe fn deref(self) -> &'a T;

    /// 返回包含值的拷贝。
    ///
    /// # 安全性
    /// - 此时该 `UnsafeCell` 不得存在指向其内容的可变引用（否则可能违反别名/可变性规则）。
    /// - 必须避免数据竞争：若多个线程可访问同一 `UnsafeCell`，所有写入必须有恰当的 happens-before 关系或使用原子操作。
    unsafe fn read(self) -> T
    where
        T: Copy;
}

impl<'a, T> UnsafeCellDeref<'a, T> for &'a UnsafeCell<T> {
    #[inline]
    unsafe fn deref_mut(self) -> &'a mut T {
        unsafe { &mut *self.get() }
    }
    #[inline]
    unsafe fn deref(self) -> &'a T {
        unsafe { &*self.get() }
    }

    #[inline]
    unsafe fn read(self) -> T
    where
        T: Copy,
    {
        unsafe { self.get().read() }
    }
}

/// 安全的将一个带所有权的值交给 [`MovingPtr`] 托管并最小化栈区拷贝
///
/// 此宏只能用作语句而非表达式，它做了两节事：
/// 1. 将对象移动到宏所在的作用域
/// 2. 创建一个指向它的 MovingPtr 指针并托管其 `Drop::drop`
#[macro_export]
macro_rules! move_as_ptr {
    ($value: ident) => {
        let mut $value = core::mem::MaybeUninit::new($value);
        let $value = unsafe { $crate::MovingPtr::from_value(&mut $value) };
    };
}

/// [`deconstruct_moving_ptr`] 的复制宏
#[macro_export]
#[doc(hidden)]
macro_rules! get_pattern {
    ($field_index:tt) => {
        $field_index
    };
    ($field_index:tt: $pattern:pat) => {
        $pattern
    };
}

/// 将一个 [`MovingPtr`] 解构为其各个字段。
///
/// 该宏会消耗原始的 [`MovingPtr`] 并为每个字段生成指向该字段的 [`MovingPtr`] 包装器。被解构的值不会被丢弃。
///
/// 宏应包裹一个带有结构体模式的 `let` 表达式。它不支持按位置匹配元组的语法，
/// 对于元组结构体请使用 `0: pat` 形式的字段匹配。
///
/// 若要对元组本身进行解构，请传入标识符 `tuple`，例如：
/// `let tuple { 0: pat0, 1: pat1 } = value`。
///
/// 该宏亦可对 `MaybeUninit` 进行投影：
/// 将类型名或 `tuple` 用 `MaybeUninit::<_>` 包裹，则宏会把 `MovingPtr<MaybeUninit<Parent>>`
/// 解构为对应的 `MovingPtr<MaybeUninit<Field>>` 值。
///
/// # Examples
///
/// ## Structs
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vct_ptr::{MovingPtr, move_as_ptr};
/// # use vct_ptr::Unaligned;
/// # struct FieldAType(usize);
/// # struct FieldBType(usize);
/// # struct FieldCType(usize);
///
/// # pub struct Parent {
/// #  pub field_a: FieldAType,
/// #  pub field_b: FieldBType,
/// #  pub field_c: FieldCType,
/// # }
///
/// let parent = Parent {
///   field_a: FieldAType(11),
///   field_b: FieldBType(22),
///   field_c: FieldCType(33),
/// };
///
/// let mut target_a = FieldAType(101);
/// let mut target_b = FieldBType(102);
/// let mut target_c = FieldCType(103);
///
/// // 将 `parent` 转换为 `MovingPtr`
/// move_as_ptr!(parent);
///
/// // 字段名需与类型定义中的名称一致。
/// // 宏会为每个字段生成对应类型的 `MovingPtr`。
/// vct_ptr::deconstruct_moving_ptr!({
///   let Parent { field_a, field_b, field_c } = parent;
/// });
///
/// field_a.assign_to(&mut target_a);
/// field_b.assign_to(&mut target_b);
/// field_c.assign_to(&mut target_c);
///
/// assert_eq!(target_a.0, 11);
/// assert_eq!(target_b.0, 22);
/// assert_eq!(target_c.0, 33);
/// ```
///
/// ## Tuples
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vct_ptr::{MovingPtr, move_as_ptr};
/// # use vct_ptr::Unaligned;
/// # struct FieldAType(usize);
/// # struct FieldBType(usize);
/// # struct FieldCType(usize);
///
/// # pub struct Parent {
/// #   pub field_a: FieldAType,
/// #  pub field_b: FieldBType,
/// #  pub field_c: FieldCType,
/// # }
///
/// let parent = (
///   FieldAType(11),
///   FieldBType(22),
///   FieldCType(33),
/// );
///
/// let mut target_a = FieldAType(101);
/// let mut target_b = FieldBType(102);
/// let mut target_c = FieldCType(103);
///
/// // 将 `parent` 转换为 `MovingPtr`
/// move_as_ptr!(parent);
///
/// // 字段模式需使用索引形式。
/// vct_ptr::deconstruct_moving_ptr!({
///   let tuple { 0: field_a, 1: field_b, 2: field_c } = parent;
/// });
///
/// field_a.assign_to(&mut target_a);
/// field_b.assign_to(&mut target_b);
/// field_c.assign_to(&mut target_c);
///
/// assert_eq!(target_a.0, 11);
/// assert_eq!(target_b.0, 22);
/// assert_eq!(target_c.0, 33);
/// ```
///
/// ## `MaybeUninit`
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vct_ptr::{MovingPtr, move_as_ptr};
/// # use vct_ptr::Unaligned;
/// # struct FieldAType(usize);
/// # struct FieldBType(usize);
/// # struct FieldCType(usize);
///
/// # pub struct Parent {
/// #  pub field_a: FieldAType,
/// #  pub field_b: FieldBType,
/// #  pub field_c: FieldCType,
/// # }
///
/// let parent = MaybeUninit::new(Parent {
///   field_a: FieldAType(11),
///   field_b: FieldBType(22),
///   field_c: FieldCType(33),
/// });
///
/// let mut target_a = MaybeUninit::new(FieldAType(101));
/// let mut target_b = MaybeUninit::new(FieldBType(102));
/// let mut target_c = MaybeUninit::new(FieldCType(103));
///
/// // 将 `parent` 转换为 `MovingPtr`
/// move_as_ptr!(parent);
///
/// // 字段名需与类型定义中的名称一致，宏会生成 `MaybeUninit` 版本的 `MovingPtr`。
/// vct_ptr::deconstruct_moving_ptr!({
///   let MaybeUninit::<Parent> { field_a, field_b, field_c } = parent;
/// });
///
/// field_a.assign_to(&mut target_a);
/// field_b.assign_to(&mut target_b);
/// field_c.assign_to(&mut target_c);
///
/// unsafe {
///   assert_eq!(target_a.assume_init().0, 11);
///   assert_eq!(target_b.assume_init().0, 22);
///   assert_eq!(target_c.assume_init().0, 33);
/// }
/// ```
///
/// [`assign_to`]: MovingPtr::assign_to
#[macro_export]
macro_rules! deconstruct_moving_ptr {
    ({ let tuple { $($field_index:tt: $pattern:pat),* $(,)? } = $ptr:expr ;}) => {
        let mut ptr: $crate::MovingPtr<_, _> = $ptr;
        let _ = || {
            let value = &mut *ptr;
            core::hint::black_box(($(&mut value.$field_index,)*));
            fn unreachable<T>(_index: usize) -> T {
                unreachable!()
            }
            *value = ($(unreachable($field_index),)*);
        };
        $(let $pattern = unsafe { ptr.move_field(|f| &raw mut (*f).$field_index) };)*
        core::mem::forget(ptr);
    };
    ({ let MaybeUninit::<tuple> { $($field_index:tt: $pattern:pat),* $(,)? } = $ptr:expr ;}) => {
        let mut ptr: $crate::MovingPtr<core::mem::MaybeUninit<_>, _> = $ptr;
        let _ = || {
            let value = unsafe { ptr.assume_init_mut() };
            core::hint::black_box(($(&mut value.$field_index,)*));
            fn unreachable<T>(_index: usize) -> T {
                unreachable!()
            }
            *value = ($(unreachable($field_index),)*);
        };
        $(let $pattern = unsafe { ptr.move_maybe_uninit_field(|f| &raw mut (*f).$field_index) };)*
        core::mem::forget(ptr);
    };
    ({ let $struct_name:ident { $($field_index:tt$(: $pattern:pat)?),* $(,)? } = $ptr:expr ;}) => {
        let mut ptr: $crate::MovingPtr<_, _> = $ptr;
        let _ = || {
            let value = &mut *ptr;
            let $struct_name { $($field_index: _),* } = value;
            core::hint::black_box(($(&mut value.$field_index),*));
            let value: *mut _ = value;
            $struct_name { ..unsafe { value.read() } };
        };
        $(let $crate::get_pattern!($field_index$(: $pattern)?) = unsafe { ptr.move_field(|f| &raw mut (*f).$field_index) };)*
        core::mem::forget(ptr);
    };
    ({ let MaybeUninit::<$struct_name:ident> { $($field_index:tt$(: $pattern:pat)?),* $(,)? } = $ptr:expr ;}) => {
        let mut ptr: $crate::MovingPtr<core::mem::MaybeUninit<_>, _> = $ptr;
        let _ = || {
            let value = unsafe { ptr.assume_init_mut() };
            let $struct_name { $($field_index: _),* } = value;
            core::hint::black_box(($(&mut value.$field_index),*));
            let value: *mut _ = value;
            $struct_name { ..unsafe { value.read() } };
        };
        $(let $crate::get_pattern!($field_index$(: $pattern)?) = unsafe { ptr.move_maybe_uninit_field(|f| &raw mut (*f).$field_index) };)*
        core::mem::forget(ptr);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    #[test]
    fn test_const_non_null() {
        // Ignore the content tested in the document.
        let x = 7u8;
        let y = 31;
        let mut z = 93i64;

        let ptr_x = ConstNonNull::from(NonNull::from(&x));
        let ptr_y = ConstNonNull::from(&y);
        let ptr_z = ConstNonNull::from(&mut z);

        assert_eq!(unsafe { *ptr_x.as_ref() }, 7u8);
        assert_eq!(unsafe { *ptr_y.as_ref() }, 31);
        assert_eq!(unsafe { *ptr_z.as_ref() }, 93i64);
    }

    #[test]
    fn test_debug_ensure_aligned() {
        let mut x = 44;
        let p = &raw mut x;
        assert_eq!(p.debug_ensure_aligned(), &raw mut x);
    }

    #[test]
    fn test_ptr() {
        let x = 7u8;
        let px = unsafe { Ptr::<'_, Aligned>::new(NonNull::from(&x).cast()) };
        assert_eq!(unsafe { *px.deref::<u8>() }, x);

        let y = 71;
        let addr = (&raw const y) as usize;
        let py = Ptr::from(&y);
        assert_eq!(py.as_ptr() as usize, addr);

        let _ = unsafe { py.into_mut() };
        let _ = NonNull::<u8>::from(py);
        let _ = py.to_unaligned();

        let py = unsafe { py.byte_add(4).byte_offset(-4) };
        assert_eq!(unsafe { *py.deref::<i32>() }, y);

        let z = 123u32;
        let aligned = Ptr::from(&z);
        // Pointer formatting should match the raw pointer
        assert_eq!(format!("{:p}", aligned), format!("{:p}", aligned.as_ptr()));
        // Debug output should include the type + alignment tag
        assert!(format!("{:?}", aligned).contains("Ptr<Aligned>"));

        let unaligned = aligned.to_unaligned();
        assert_eq!(
            format!("{:p}", unaligned),
            format!("{:p}", unaligned.as_ptr())
        );
        assert!(format!("{:?}", unaligned).contains("Ptr<Unaligned>"));
    }

    #[test]
    fn test_ptr_mut() {
        let mut x = 7u8;
        let px = unsafe { PtrMut::<'_, Aligned>::new(NonNull::from(&mut x).cast()) };
        assert_eq!(unsafe { *px.deref_mut::<u8>() }, x);

        let mut y = 71;
        let addr = (&raw const y) as usize;
        let mut py = PtrMut::from(&mut y);
        assert_eq!(py.as_ptr() as usize, addr);

        let _ = py.as_ref();
        let _ = py.reborrow();
        let _ = unsafe { py.promote() };
        let py = PtrMut::from(&mut y);
        let _ = NonNull::<u8>::from(py);
        let py = PtrMut::from(&mut y);
        let _ = py.to_unaligned();

        let py = PtrMut::from(&mut y);
        let py = unsafe { py.byte_add(4).byte_offset(-4) };
        assert_eq!(unsafe { *py.deref_mut::<i32>() }, y);

        let mut z = 123u32;
        let aligned = PtrMut::from(&mut z);
        // Pointer formatting should match the raw pointer
        assert_eq!(format!("{:p}", aligned), format!("{:p}", aligned.as_ptr()));
        // Debug output should include the type + alignment tag
        assert!(format!("{:?}", aligned).contains("PtrMut<Aligned>"));

        let unaligned = aligned.to_unaligned();
        assert_eq!(
            format!("{:p}", unaligned),
            format!("{:p}", unaligned.as_ptr())
        );
        assert!(format!("{:?}", unaligned).contains("PtrMut<Unaligned>"));
    }

    #[test]
    fn test_owning_ptr() {
        let mut x = 7u8;
        let px = unsafe { OwningPtr::<'_, Aligned>::new(NonNull::from(&mut x).cast()) };
        assert_eq!(unsafe { px.read::<u8>() }, x);

        let y: i32 = 71;
        OwningPtr::make(y, |py| {
            let mut py = py;
            {
                let p1 = py.as_ref();
                assert_eq! {unsafe{ *p1.deref::<i32>() }, y};
            }
            {
                let p2 = py.as_mut();
                unsafe {
                    *p2.deref_mut::<i32>() += 3;
                }
            }
            assert_eq!(unsafe { py.read::<i32>() }, 74);
        });

        let mut z = 123u32;
        let aligned = unsafe { PtrMut::from(&mut z).promote() };
        // Pointer formatting should match the raw pointer
        assert_eq!(format!("{:p}", aligned), format!("{:p}", aligned.as_ptr()));
        // Debug output should include the type + alignment tag
        assert!(format!("{:?}", aligned).contains("OwningPtr<Aligned>"));

        let unaligned = aligned.to_unaligned();
        assert_eq!(
            format!("{:p}", unaligned),
            format!("{:p}", unaligned.as_ptr())
        );
        assert!(format!("{:?}", unaligned).contains("OwningPtr<Unaligned>"));
    }

    #[test]
    fn test_moving_ptr() {
        // read from value
        let mut v1 = MaybeUninit::new(10u32);
        let mp1 = unsafe { MovingPtr::from_value(&mut v1) };
        let r1 = mp1.read();
        assert_eq!(r1, 10);

        // write_to (doesn't drop dst)
        let mut v2 = MaybeUninit::new(11u32);
        let mp2 = unsafe { MovingPtr::from_value(&mut v2) };
        let mut dst: u32 = 0;
        unsafe { mp2.write_to(&raw mut dst) };
        assert_eq!(dst, 11);

        // assign_to (drops dst then writes)
        let mut v3 = MaybeUninit::new(12u32);
        let mp3 = unsafe { MovingPtr::from_value(&mut v3) };
        let mut target: u32 = 1;
        mp3.assign_to(&mut target);
        assert_eq!(target, 12);

        // to_unaligned + TryFrom back to Aligned
        let mut v4 = MaybeUninit::new(20u32);
        let mp4 = unsafe { MovingPtr::from_value(&mut v4) };
        let mp_un = mp4.to_unaligned();
        let res = MovingPtr::<u32, Aligned>::try_from(mp_un);
        assert!(res.is_ok());
        let mp5 = res.unwrap();
        let r5 = mp5.read();
        assert_eq!(r5, 20);

        struct DropCounter<'a>(&'a Cell<u32>);
        impl<'a> Drop for DropCounter<'a> {
            fn drop(&mut self) {
                let c = self.0.get();
                self.0.set(c + 1);
            }
        }

        let counter = Cell::new(0u32);
        let mut v = MaybeUninit::new(DropCounter(&counter));
        {
            // create a MovingPtr that will drop the inner value when dropped
            let _mp = unsafe { MovingPtr::from_value(&mut v) };
            // `_mp` goes out of scope here -> Drop should run
        }
        assert_eq!(counter.get(), 1);
    }
}
