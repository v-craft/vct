#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
#![expect(unsafe_code, reason = "Raw pointers are inherently unsafe.")]

// use alloc for test
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

/// A read-only [`NonNull<T>`].
///
/// Can only directly obtain immutable references.
#[repr(transparent)]
pub struct ConstNonNull<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> ConstNonNull<T> {
    /// Creates a new `ConstNonNull` if `ptr` is non-null.
    ///
    /// # Example
    ///
    /// ```
    /// use vct_ptr::ConstNonNull;
    ///
    /// let x = 0u32;
    /// let ptr = ConstNonNull::new(&raw const x).expect("ptr is null!");
    ///
    /// if let Some(_) = ConstNonNull::<u32>::new(core::ptr::null()) {
    ///     unreachable!();
    /// }
    /// ```
    #[inline]
    pub const fn new(ptr: *const T) -> Option<Self> {
        // NonNull::new(ptr.cast_mut()).map(Self)
        // `map` is not stable const fn
        match NonNull::new(ptr.cast_mut()) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    /// Creates a new `ConstNonNull`, `ptr` must be non-null.
    ///
    /// It's UB if `ptr` is non-null. ⚠️
    ///
    /// # Example
    ///
    /// ```
    /// use vct_ptr::ConstNonNull;
    ///
    /// let x = 0u32;
    /// let ptr = unsafe { ConstNonNull::new_unchecked(&raw const x) };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
        // Safety: `ptr` must be non-null.
        unsafe { Self(NonNull::new_unchecked(ptr.cast_mut())) }
    }

    /// Returns a immutable reference to the value.
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that all of the following is true:
    ///
    /// - The pointer must be [properly aligned].
    /// - The pointer must point to an initialized instance of `T`.
    /// - It must be "dereferenceable".
    /// - Enforce Rust's aliasing rules.
    ///
    /// This applies even if the result of this method is unused!
    ///
    /// More details: <https://doc.rust-lang.org/core/ptr/index.html>
    ///
    /// # Example
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
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn as_ref<'a>(&self) -> &'a T {
        // Safety: See `NonNull::as_ref`
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

/// Used as a type argument to [`Ptr`], [`PtrMut`], [`OwningPtr`], and [`MovingPtr`]
/// to specify that the pointer is guaranteed to be [aligned].
///
/// [aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Debug, Copy, Clone)]
pub struct Aligned;

/// Used as a type argument to [`Ptr`], [`PtrMut`], [`OwningPtr`], and [`MovingPtr`]
/// to specify that the pointer may not [aligned].
///
/// [aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Debug, Copy, Clone)]
pub struct Unaligned;

// Seal traits and prohibit external impl
mod seal_aligned {
    pub trait Sealed {}
    impl Sealed for super::Aligned {}
    impl Sealed for super::Unaligned {}
}

/// Provide some pointer operations.
///
/// Only implemented for [`Aligned`] and [`Unaligned`].
pub trait IsAligned: seal_aligned::Sealed {
    /// Reads the value pointed to by `ptr`.
    ///
    /// # Safety
    ///  - `ptr` must be valid for reads.
    ///  - `ptr` must point to a valid instance of type `T`
    ///  - If this type is [`Aligned`], then `ptr` must be properly aligned for type `T`.
    unsafe fn read_ptr<T>(ptr: *const T) -> T;

    /// Copies `count * size_of::<T>()` bytes from `src` to `dst`. The source
    /// and destination must *not* overlap.
    ///
    /// # Safety
    ///  - `src` must be valid for reads of `count * size_of::<T>()` bytes.
    ///  - `dst` must be valid for writes of `count * size_of::<T>()` bytes.
    ///  - The region of memory beginning at `src` with a size of `count *
    ///    size_of::<T>()` bytes must *not* overlap with the region of memory
    ///    beginning at `dst` with the same size.
    ///  - If this type is [`Aligned`], then both `src` and `dst` must properly
    ///    be aligned for values of type `T`.
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);

    /// Reads the value pointed to by `ptr`.
    ///
    /// # Safety
    ///  - `ptr` must be valid for reads and writes.
    ///  - `ptr` must point to a valid instance of type `T`
    ///  - If this type is [`Aligned`], then `ptr` must be properly aligned for type `T`.
    ///  - The value pointed to by `ptr` must be valid for dropping.
    ///  - While `drop_in_place` is executing, the only way to access parts of `ptr` is through
    ///    the `&mut Self` supplied to it's `Drop::drop` impl.
    unsafe fn drop_in_place<T>(ptr: *mut T);
}

impl IsAligned for Aligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        // Safety: See [`ptr::read`]
        unsafe { ptr.read() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        // Safety: See [`ptr::copy_nonoverlapping`]
        unsafe {
            ptr::copy_nonoverlapping(src, dst, count);
        }
    }

    #[inline]
    unsafe fn drop_in_place<T>(ptr: *mut T) {
        // Safety: See [`ptr::drop_in_place`]
        unsafe {
            ptr::drop_in_place(ptr);
        }
    }
}

impl IsAligned for Unaligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        // Safety: See [`ptr::read_unaligned`]
        unsafe { ptr.read_unaligned() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        // Safety: See [`ptr::copy_nonoverlapping`]
        // This is doing a byte-wise copy, always guaranteed to be aligned.
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
        // SAFETY:
        //  - `ptr` must be valid for reads and writes.
        //  - `ptr` points to a valid instance of type `T`.
        //  - `ptr` points must be valid for dropping.
        //  - `ptr` points must not be used after this function call.
        //  - This type is not `Aligned` so the caller does not need to ensure `properly aligned`.
        unsafe {
            drop(ptr.read_unaligned());
        }
    }
}

/// A type-erased pointer, similar to `&'a dyn Any`
///
/// This type tries to act "borrow-like" which means that:
/// - It must always point to a valid value of whatever the pointee type is.
/// - Immutable: its target must not be changed while this pointer is alive.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must be [properly aligned] for underlying type.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Ptr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a u8, A)>);

/// A type-erased pointer, similar to `&'a mut dyn Any`
///
/// This type tries to act "borrow-like" which means that:
/// - It must always point to a valid value of whatever the pointee type is.
/// - Exclusive and Mutable: It cannot be cloned, and the caller must comply with Rust alias rules.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must be [properly aligned] for underlying type.
#[repr(transparent)]
pub struct PtrMut<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// A type-erased pointer, similar to `&'a mut ManuallyDrop<dyn Any>`
///
/// Conceptually represents ownership of whatever data is being pointed to
/// and so is responsible for calling its [`Drop`] impl.
///
/// This pointer is **not** responsible for freeing the memory pointed to by this pointer
/// as it may be pointing to an element in a `Vec` or to a local in a function etc.
///
/// This type tries to act "borrow-like" which means that:
/// - It must always point to a valid value of whatever the pointee type is.
/// - Exclusive and Mutable: It cannot be cloned, and the caller must comply with Rust alias rules.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must be [properly aligned] for underlying type.
#[repr(transparent)]
pub struct OwningPtr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// A pointer for moving value, similar to `&'a mut ManuallyDrop<T>`
///
/// Conceptually represents ownership of whatever data is being pointed to
/// and will **auto** call its [`Drop`] impl when self be dropped.
///
/// This pointer is **not** responsible for freeing the memory pointed to by this pointer
/// as it may be pointing to an element in a `Vec` or to a local in a function etc.
///
/// Referring to C++ `std::move`. A small object is responsible for managing large objects memory,
/// and this pointer is used to host the ['Drop'] impl of small objects.
/// So you can **move(copy)** small object by this ptr and don't trigger ['Drop'] impl.
///
/// This type tries to act "borrow-like" which means that:
/// - It must always point to a valid value of whatever the pointee type is.
/// - Exclusive and Mutable: It cannot be cloned, and the caller must comply with Rust alias rules.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must be [properly aligned] for underlying type.
#[repr(transparent)]
pub struct MovingPtr<'a, T, A: IsAligned = Aligned>(NonNull<T>, PhantomData<(&'a mut T, A)>);

/// For checking pointer alignment in debug mode.
trait DebugEnsureAligned {
    fn debug_ensure_aligned(self) -> Self;
}

macro_rules! impl_debug_aligned_for_ptr {
    ($mutablity:ident) => {
        // miri runs with built-in checks.
        #[cfg(all(debug_assertions, not(miri)))]
        impl<T: Sized> DebugEnsureAligned for *$mutablity T {
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
        impl<T: Sized> DebugEnsureAligned for *$mutablity T {
            #[inline(always)]
            fn debug_ensure_aligned(self) -> Self {
                self
            }
        }
    };
}

impl_debug_aligned_for_ptr!(mut);
impl_debug_aligned_for_ptr!(const);

impl<'a, A: IsAligned> Ptr<'a, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value.
    /// - If it's [`Aligned`], `inner` must be properly aligned for the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`Ptr`] will stay valid.
    /// - Nothing can mutate the pointee while this [`Ptr`] is live except through an [`UnsafeCell`].
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets the underlying pointer, erasing the associated lifetime.
    ///
    /// # Safety:
    /// - If pointee is immutable, could not use this ptr to modify it.
    /// - After the pointee is modified, `Ptr` self is no longer available.
    ///
    /// If possible, it is encouraged to use [`deref`](Self::deref) over this function.
    #[inline]
    pub const fn as_ptr(self) -> *const u8 {
        self.0.as_ptr()
    }

    /// Transforms this [`Ptr<T>`] into a `&T` with the same lifetime
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`Ptr`].
    /// - Self must be properly aligned for the pointee type `T`.
    #[inline]
    pub unsafe fn deref<T>(self) -> &'a T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        // Safety: Type correct, ptr aligned and pointee valid object.
        unsafe { &*ptr }
    }

    /// Transforms this [`Ptr`] into an [`PtrMut`]
    ///
    /// # Safety
    /// - The data pointed to by this `Ptr` must be valid for writes.
    /// - There must be no active references (mutable or otherwise) to the data underlying this `Ptr`.
    /// - Self could not be used after this function call.
    /// - Another [`PtrMut`] for the same [`Ptr`] must not be created until the first is dropped.
    #[inline]
    pub const unsafe fn into_mut(self) -> PtrMut<'a, A> {
        PtrMut(self.0, PhantomData)
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ptr<'a> {
    #[inline]
    fn from(val: &'a T) -> Self {
        // manually inline, istead of `Ptr::new`
        Self(NonNull::from_ref(val).cast(), PhantomData)
    }
}

impl<'a, A: IsAligned> PtrMut<'a, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value.
    /// - If it's [`Aligned`], `inner` must be properly aligned for the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`Ptr`] will stay valid.
    /// - Nothing else can read or mutate the pointee while this [`PtrMut`] is live.
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets the underlying pointer, erasing the associated lifetime.
    ///
    /// If possible, it is encouraged to use [`deref_mut`](Self::deref_mut).
    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// Transforms this [`PtrMut<T>`] into a `&mut T` with the same lifetime
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`PtrMut`].
    /// - If the type parameter `A` is [`Unaligned`] then this pointer must be [properly aligned]
    ///   for the pointee type `T`.
    #[inline]
    pub unsafe fn deref_mut<T>(self) -> &'a mut T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        // Safety: Type correct, ptr aligned and pointee valid object.
        unsafe { &mut *ptr }
    }

    /// Gets an immutable reference from this mutable reference
    ///
    /// # Safety
    /// - During the use of Ptr, the original PtrMut is unavailable.
    ///
    /// # Example
    ///
    /// ```
    /// # use vct_ptr::{PtrMut, Ptr};
    /// #
    /// let mut x = 5;
    /// let mut pm = PtrMut::from(&mut x);
    ///
    /// // When the new pointer is valid,
    /// // the original pointer is unavailable.
    /// foo(pm.as_ref());
    ///
    /// fn foo(ptr: Ptr<'_>) {
    ///     // Safe:
    ///     // When the reborrowed ptr is valid,
    ///     // the original PtrMut is unavailable.
    /// }
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Ptr<'_, A> {
        // Safety: See function docs
        unsafe { Ptr::new(self.0) }
    }

    /// Gets a [`PtrMut`] from this with a smaller lifetime.
    ///
    /// # Safety
    /// - There can only be one active mutable pointer at a time.
    ///
    /// # Example
    ///
    /// ```
    /// use vct_ptr::PtrMut;
    ///
    /// let mut x = 5;
    /// let mut pm = PtrMut::from(&mut x);
    ///
    /// // When the new pointer is valid,
    /// // the original pointer is unavailable.
    /// foo(pm.reborrow());
    ///
    /// fn foo(ptr: PtrMut<'_>) {
    ///     // Safe:
    ///     // When the reborrowed ptr is valid,
    ///     // the original PtrMut is unavailable.
    /// }
    /// ```
    ///
    /// Use `&mut self` for check the safety of borrowing.
    #[inline]
    pub const fn reborrow(&mut self) -> PtrMut<'_, A> {
        // Safety: See function docs
        unsafe { PtrMut::new(self.0) }
    }

    /// Transforms this [`PtrMut`] into an [`OwningPtr`]
    ///
    /// # Safety
    /// Must have right to drop or move out of [`PtrMut`].
    /// - This function does not cancel the automatic 'drop' of the pointee.
    /// - You may need to manually convert the pointee `T` to `ManuallyDrop<T>`.
    /// - And then, remember to manually call pointee's `Drop` impl it when needed.
    ///
    /// So, the pointee type is usually required to be `ManuallyDrop<T>``.
    ///
    /// # Example
    ///
    /// ```
    /// # use vct_ptr::{PtrMut, OwningPtr};
    /// # use core::mem::ManuallyDrop;
    /// #
    /// let mut data = ManuallyDrop::new(312);
    /// let ptr = unsafe{ PtrMut::from(&mut data).promote() };
    /// ```
    #[inline]
    pub const unsafe fn promote(self) -> OwningPtr<'a, A> {
        OwningPtr(self.0, PhantomData)
    }
}

impl<'a, T: ?Sized> From<&'a mut T> for PtrMut<'a> {
    #[inline]
    fn from(val: &'a mut T) -> Self {
        // manually inline, istead of `Ptr::new`
        Self(NonNull::from_mut(val).cast(), PhantomData)
    }
}

impl<'a> OwningPtr<'a> {
    /// Consumes a value and creates an [`OwningPtr`] to it
    /// while ensuring a double drop does not happen.
    ///
    /// # Safety
    /// - OwningPtr should be consumed in function `f`.
    /// - `Drop` impl should be manually called.
    #[inline]
    pub fn make<T, F: FnOnce(OwningPtr<'_>) -> R, R>(val: T, f: F) -> R {
        let mut val = ManuallyDrop::new(val);
        f(Self(NonNull::from_mut(&mut val).cast(), PhantomData))
        // f(unsafe{ PtrMut::from(temp).promote() })
    }
}

impl<'a> OwningPtr<'a, Unaligned> {
    /// Consumes the [`OwningPtr`] to obtain ownership of the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`OwningPtr`].
    #[inline]
    pub unsafe fn read_unaligned<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>();
        unsafe { ptr.read_unaligned() }
    }
}

impl<'a, A: IsAligned> OwningPtr<'a, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value of whatever the pointee type is.
    /// - If the `A` type parameter is [`Aligned`] then `inner` must be [properly aligned] for the pointee type.
    /// - `inner` must have correct provenance to allow read and writes of the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`OwningPtr`] will stay valid and nothing
    ///   else can read or mutate the pointee while this [`OwningPtr`] is live.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets the underlying pointer, erasing the associated lifetime.
    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// Consumes the [`OwningPtr`] to obtain ownership of the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`OwningPtr`].
    /// - If it's [`Aligned`], then this pointer must be aligned for type `T`.
    #[inline]
    pub unsafe fn read<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        // SAFETY: The caller ensure the pointee is of type `T` and uphold safety for `read`.
        unsafe { ptr.read() }
    }

    /// Consumes the [`OwningPtr`] to drop the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`OwningPtr`].
    /// - If it's [`Aligned`], then this pointer must be aligned for type `T`.
    #[inline]
    pub unsafe fn drop_as<T>(self) {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        // SAFETY: The caller ensure the pointee is of type `T` and uphold safety for `drop_in_place`.
        unsafe {
            ptr.drop_in_place();
        }
    }

    /// Gets an immutable pointer from this owned pointer.
    ///
    /// # Safety
    /// See [`PtrMut::as_ref`]
    #[inline]
    pub const fn as_ref(&self) -> Ptr<'_, A> {
        // Safety: See [`PtrMut::as_ref`]
        unsafe { Ptr::new(self.0) }
    }

    /// Gets a mutable pointer from this owned pointer.
    ///
    /// # Safety
    /// See [`PtrMut::reborrow`]
    #[inline]
    pub const fn as_mut(&mut self) -> PtrMut<'_, A> {
        // Safety: See [`PtrMut::reborrow`]
        unsafe { PtrMut::new(self.0) }
    }

    /// Casts to a concrete type as a [`MovingPtr`].
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`OwningPtr`].
    #[inline]
    pub const unsafe fn cast<T>(self) -> MovingPtr<'a, T, A> {
        MovingPtr(self.0.cast::<T>(), PhantomData)
    }
}

macro_rules! impl_ptr {
    ($ptr:ident) => {
        impl<'a> $ptr<'a, Aligned> {
            /// Removes the alignment requirement of this pointer
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
            /// Calculates the offset from a pointer.
            ///
            /// As the pointer is type-erased, `count` parameter is in raw bytes.
            ///
            /// # Safety
            /// - The offset cannot make the existing ptr null or invalid target.
            /// - If the `A` type parameter is [`Aligned`] then the offset must not make the
            ///   resulting pointer be unaligned.
            /// - The resulting pointer must outlive the lifetime of this pointer.
            #[inline]
            pub const unsafe fn byte_offset(self, count: isize) -> Self {
                Self(
                    // Safety: The caller upholds safety for `offset` and ensures the result is not null.s
                    unsafe { NonNull::new_unchecked(self.as_ptr().offset(count) as *mut u8) },
                    PhantomData,
                )
            }

            /// Calculates the offset from a pointer.
            ///
            /// As the pointer is type-erased, `count` parameter is in raw bytes.
            ///
            /// # Safety
            /// - The offset cannot make the existing ptr null or invalid target.
            /// - If the `A` type parameter is [`Aligned`] then the offset must not make the
            ///   resulting pointer be unaligned.
            /// - The resulting pointer must outlive the lifetime of this pointer.
            #[inline]
            pub const unsafe fn byte_add(self, count: usize) -> Self {
                Self(
                    // SAFETY: The caller upholds safety for `add` and ensures the result is not null.
                    unsafe { NonNull::new_unchecked(self.as_ptr().add(count) as *mut u8) },
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
    /// Removes the alignment requirement of this pointer
    #[inline]
    pub const fn to_unaligned(self) -> MovingPtr<'a, T, Unaligned> {
        let value = MovingPtr(self.0, PhantomData);
        mem::forget(self);
        value
    }

    /// Creates a [`MovingPtr`] from a provided value of type `T`.
    ///
    /// For a safer alternative, it is advised to use [`move_as_ptr`] where possible.
    ///
    /// # Safety
    /// - `value` must store a properly initialized value of type `T`.
    /// - Once the returned [`MovingPtr`] has been used, `value` must be treated as
    ///   it were uninitialized unless it was explicitly leaked via [`core::mem::forget`].
    #[inline]
    pub unsafe fn from_value(value: &'a mut MaybeUninit<T>) -> Self {
        MovingPtr(NonNull::from_mut(value).cast::<T>(), PhantomData)
    }
}

impl<'a, T, A: IsAligned> MovingPtr<'a, T, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// For a safer alternative, it is advised to use [`move_as_ptr`] where possible.
    ///
    /// # Safety
    /// - `inner` must point to valid value of `T`.
    /// - If the `A` type parameter is [`Aligned`] then `inner` must be aligned for `T`.
    /// - `inner` must have correct provenance to allow read and writes of the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`MovingPtr`] will stay valid and nothing
    ///   else can read or mutate the pointee while this [`MovingPtr`] is live.
    #[inline]
    pub const unsafe fn new(inner: NonNull<T>) -> Self {
        Self(inner, PhantomData)
    }

    /// Partially moves out some fields inside of `self`.
    ///
    /// The partially returned value is returned back pointing to [`MaybeUninit<T>`].
    ///
    /// While calling this function is safe, care must be taken with the returned `MovingPtr` as it
    /// points to a value that may no longer be completely valid.
    ///
    /// # Example
    ///
    /// ```
    /// use core::mem::{offset_of, MaybeUninit, forget};
    /// use vct_ptr::{MovingPtr, move_as_ptr};
    /// # struct FieldAType(usize);
    /// # struct FieldBType(usize);
    /// # struct FieldCType(usize);
    /// fn insert<T>(_ptr: MovingPtr<'_, T>) { /*...*/ }
    ///
    /// struct Parent {
    ///   field_a: FieldAType,
    ///   field_b: FieldBType,
    ///   field_c: FieldCType,
    /// }
    ///
    /// let parent = Parent {
    ///   field_a: FieldAType(0),
    ///   field_b: FieldBType(0),
    ///   field_c: FieldCType(0),
    /// };
    ///
    /// // Moving `parent` to `Maybeuninit` in this scope,
    /// // and create a `MovingPtr` with the same name.
    /// move_as_ptr!(parent);
    ///
    /// let (partial_parent, ()) = parent.partial_move(|parent_ptr| unsafe {
    ///     vct_ptr::deconstruct_moving_ptr!({
    ///         let Parent { field_a, field_b, field_c } = parent_ptr;
    ///     });
    ///
    ///     insert(field_a);
    ///     insert(field_b);
    ///     forget(field_c);
    /// });
    ///
    /// unsafe {
    ///     vct_ptr::deconstruct_moving_ptr!({
    ///         let MaybeUninit::<Parent> { field_a: _, field_b: _, field_c } = partial_parent;
    ///     });
    ///
    ///     insert(field_c);
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

    /// Reads the value pointed to by this pointer.
    #[inline]
    pub fn read(self) -> T {
        // SAFETY:
        //  - `self.0` must be valid for reads as this type owns the value it points to.
        //  - `self.0` must always point to a valid instance of type `T`
        //  - If `A` is [`Aligned`], then `ptr` must be properly aligned for type `T`.
        let value = unsafe { A::read_ptr(self.0.as_ptr()) };
        mem::forget(self);
        value
    }

    /// Writes the value pointed to by this pointer to a provided location.
    ///
    /// This does **not** drop the value stored at `dst` and it's the caller's responsibility
    /// to ensure that it's properly dropped.
    ///
    /// # Safety
    ///  - `dst` must be valid for writes.
    ///  - If the `A` type parameter is [`Aligned`] then `dst` must be [properly aligned] for `T`.
    #[inline]
    pub unsafe fn write_to(self, dst: *mut T) {
        let src = self.0.as_ptr();
        mem::forget(self);
        // SAFETY: See function docs
        unsafe { A::copy_nonoverlapping(src, dst, 1) };
    }

    /// Writes the value pointed to by this pointer into `dst`.
    ///
    /// The value previously stored at `dst` will be dropped.
    #[inline]
    pub fn assign_to(self, dst: &mut T) {
        // SAFETY:
        // - `dst` is a mutable borrow,
        // - `dst` must point to a valid instance of `T`.
        // - `dst` must point to value that is valid for dropping.
        // - `dst` must not alias any other access.
        // - `dst` must be valid for writes.
        // - `dst` must always be aligned.
        unsafe {
            ptr::drop_in_place(dst);
            self.write_to(dst);
        }
    }

    /// Creates a [`MovingPtr`] for a specific field within `self`.
    ///
    /// This function is explicitly made for deconstructive moves.
    ///
    /// The correct `byte_offset` for a field can be obtained via [`core::mem::offset_of`].
    ///
    /// # Safety
    ///  - `f` must return a non-null pointer to a valid field inside `T`
    ///  - If `A` is [`Aligned`], then `T` must not be `repr(packed)`
    ///  - `self` should not be accessed or dropped as if it were a complete value after this function returns.
    ///    Other fields that have not been moved out of may still be accessed or dropped separately.
    ///  - This function cannot alias the field with any other access, including other calls to [`move_field`]
    ///    for the same field, without first calling [`forget`] on it first.
    ///
    /// A result of the above invariants means that any operation that could cause `self` to be dropped while
    /// the pointers to the fields are held will result in undefined behavior. This requires extra caution
    /// around code that may panic. See the example below for an example of how to safely use this function.
    ///
    /// # Example
    ///
    /// ```
    /// use core::mem::offset_of;
    /// use vct_ptr::{MovingPtr, move_as_ptr};
    /// # struct FieldAType(usize);
    /// # struct FieldBType(usize);
    /// # struct FieldCType(usize);
    /// fn insert<T>(_ptr: MovingPtr<'_, T>) { /*...*/ }
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
    /// // Converts `parent` into a `MovingPtr`.
    /// move_as_ptr!(parent);
    ///
    /// unsafe {
    ///    let field_a = parent.move_field(|ptr| &raw mut (*ptr).field_a);
    ///    let field_b = parent.move_field(|ptr| &raw mut (*ptr).field_b);
    ///    let field_c = parent.move_field(|ptr| &raw mut (*ptr).field_c);
    ///    // Each call to insert may panic!
    ///    // Ensure that `parent_ptr` cannot be dropped before calling them!
    ///    core::mem::forget(parent);
    ///    insert(field_a);
    ///    insert(field_b);
    ///    insert(field_c);
    /// }
    /// ```
    ///
    /// [`forget`]: core::mem::forget
    #[inline(always)]
    pub unsafe fn move_field<U>(&self, f: impl Fn(*mut T) -> *mut U) -> MovingPtr<'a, U, A> {
        MovingPtr(
            // SAFETY: The caller must ensure that `U` is the correct type for the field at `byte_offset`.
            unsafe { NonNull::new_unchecked(f(self.0.as_ptr())) },
            PhantomData,
        )
    }
}

impl<'a, T, A: IsAligned> MovingPtr<'a, MaybeUninit<T>, A> {
    /// Creates a [`MovingPtr`] for a specific field within `self`.
    ///
    /// This function is explicitly made for deconstructive moves.
    ///
    /// The correct `byte_offset` for a field can be obtained via [`core::mem::offset_of`].
    ///
    /// # Safety
    ///  - `f` must return a non-null pointer to a valid field inside `T`
    ///  - If `A` is [`Aligned`], then `T` must not be `repr(packed)`
    ///  - `self` should not be accessed or dropped as if it were a complete value after this function returns.
    ///    Other fields that have not been moved out of may still be accessed or dropped separately.
    ///  - This function cannot alias the field with any other access, including other calls to [`move_field`]
    ///    for the same field, without first calling [`forget`] on it first.
    ///
    /// [`forget`]: core::mem::forget
    /// [`move_field`]: Self::move_field
    #[inline(always)]
    pub unsafe fn move_maybe_uninit_field<U>(
        &self,
        f: impl Fn(*mut T) -> *mut U,
    ) -> MovingPtr<'a, MaybeUninit<U>, A> {
        let self_ptr = self.0.as_ptr().cast::<T>();
        // SAFETY:
        // - The caller must ensure that `U` is the correct type for the field at `byte_offset` and thus
        //   cannot be null.
        // - `MaybeUninit<T>` is `repr(transparent)` and thus must have the same memory layout as `T``
        let field_ptr = unsafe { NonNull::new_unchecked(f(self_ptr)) };
        MovingPtr(field_ptr.cast::<MaybeUninit<U>>(), PhantomData)
    }

    /// Creates a [`MovingPtr`] pointing to a valid instance of `T`.
    ///
    /// See also: [`MaybeUninit::assume_init`].
    ///
    /// # Safety
    /// It's up to the caller to ensure that the value pointed to by `self`
    /// is really in an initialized state. Calling this when the content is not yet
    /// fully initialized causes immediate undefined behavior.
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
        // SAFETY: this pointer must be aligned.
        unsafe { &*ptr }
    }
}

impl<T> DerefMut for MovingPtr<'_, T, Aligned> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.0.as_ptr().debug_ensure_aligned();
        // SAFETY: this pointer must be aligned.
        unsafe { &mut *ptr }
    }
}

impl<T, A: IsAligned> Drop for MovingPtr<'_, T, A> {
    fn drop(&mut self) {
        // SAFETY:
        //  - `self.0` must be valid for reads and writes as this pointer type owns the value it points to.
        //  - `self.0` must always point to a valid instance of type `T`
        //  - If `A` is `Aligned`, then `ptr` must be properly aligned for type `T` by construction.
        //  - `self.0` owns the value it points to so it must always be valid for dropping until this pointer is dropped.
        //  - This type owns the value it points to, so it's required to not mutably alias value that it points to.
        unsafe { A::drop_in_place(self.0.as_ptr()) };
    }
}

/// A slice like '&'A [T]', without length information for better performance
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    _marker: PhantomData<&'a [T]>,
    ptr: NonNull<T>,
    #[cfg(debug_assertions)]
    len: usize,
}

impl<'a, T> ThinSlicePtr<'a, T> {
    /// Indexes the slice without doing bounds checks
    ///
    /// # Safety
    /// `index` must be in-bounds.
    #[inline]
    pub const unsafe fn get(self, index: usize) -> &'a T {
        // debug_assert! Use if branch to determine whether to execute.
        // Therefore, #[cfg] is needed.
        #[cfg(debug_assertions)]
        debug_assert!(index < self.len);

        let ptr = self.ptr.as_ptr();
        // SAFETY: `index` is in-bounds so the resulting pointer is valid to dereference.
        unsafe { &*ptr.add(index) }
    }
}

impl<'a, T> From<&'a [T]> for ThinSlicePtr<'a, T> {
    #[inline]
    fn from(slice: &'a [T]) -> Self {
        let ptr = slice.as_ptr().cast_mut();
        Self {
            _marker: PhantomData,
            // SAFETY: a reference can never be null
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            #[cfg(debug_assertions)]
            len: slice.len(),
        }
    }
}

/// Creates a dangling pointer with specified alignment.
///
/// See [`NonNull::dangling`].
pub const fn dangling_with_align(align: NonZeroUsize) -> NonNull<u8> {
    debug_assert!(align.is_power_of_two(), "Alignment must be power of two.");

    unsafe { NonNull::new_unchecked(ptr::null_mut::<u8>().wrapping_add(align.get())) }
}

mod seal_unsafe_cell {
    use core::cell::UnsafeCell;
    pub trait Sealed {}
    impl<'a, T> Sealed for &'a UnsafeCell<T> {}
}

/// Extension trait for helper methods on [`UnsafeCell`]
pub trait UnsafeCellDeref<'a, T>: seal_unsafe_cell::Sealed {
    /// # Safety
    /// - The returned value must be unique and not alias any mutable or immutable references to the contents of the [`UnsafeCell`].
    /// - At all times, you must avoid data races. If multiple threads have access to the same [`UnsafeCell`], then any writes must have a proper happens-before relation to all other accesses or use atomics ([`UnsafeCell`] docs for reference).
    unsafe fn deref_mut(self) -> &'a mut T;

    /// # Safety
    /// - For the lifetime `'a` of the returned value you must not construct a mutable reference to the contents of the [`UnsafeCell`].
    /// - At all times, you must avoid data races. If multiple threads have access to the same [`UnsafeCell`], then any writes must have a proper happens-before relation to all other accesses or use atomics ([`UnsafeCell`] docs for reference).
    unsafe fn deref(self) -> &'a T;

    /// Returns a copy of the contained value.
    ///
    /// # Safety
    /// - The [`UnsafeCell`] must not currently have a mutable reference to its content.
    /// - At all times, you must avoid data races. If multiple threads have access to the same [`UnsafeCell`], then any writes must have a proper happens-before relation to all other accesses or use atomics ([`UnsafeCell`] docs for reference).
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

/// Safely converts a owned value into a [`MovingPtr`].
///
/// This cannot be used as expression and must be used as a statement.
///
/// This macro will do two things:
/// 1. Move target to `MaybeUninit<>`` in the scope of the macro..
/// 2. Create a MovingPtr with same name.
#[macro_export]
macro_rules! move_as_ptr {
    ($value: ident) => {
        let mut $value = core::mem::MaybeUninit::new($value);
        let $value = unsafe { $crate::MovingPtr::from_value(&mut $value) };
    };
}

/// Helper macro used by [`deconstruct_moving_ptr`]
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

/// Deconstructs a [`MovingPtr`] into its individual fields.
///
/// This consumes the [`MovingPtr`] and hands out [`MovingPtr`] wrappers around
/// pointers to each of its fields. The value will *not* be dropped.
///
/// The macro should wrap a `let` expression with a struct pattern.
/// It does not support matching tuples by position,
/// so for tuple structs you should use `0: pat` syntax.
///
/// For tuples themselves, pass the identifier `tuple` instead of the struct name,
/// like `let tuple { 0: pat0, 1: pat1 } = value`.
///
/// This can also project into `MaybeUninit`.
/// Wrap the type name or `tuple` with `MaybeUninit::<_>`,
/// and the macro will deconstruct a `MovingPtr<MaybeUninit<ParentType>>`
/// into `MovingPtr<MaybeUninit<FieldType>>` values.
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
/// // Converts `parent` into a `MovingPtr`
/// move_as_ptr!(parent);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `MovingPtr` of the field's type.
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
/// // Converts `parent` into a `MovingPtr`
/// move_as_ptr!(parent);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `MovingPtr` of the field's type.
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
/// // The field names must match the name used in the type definition.
/// // Each one will be a `MovingPtr` of the field's type.
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
    fn const_non_null() {
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
    fn debug_ensure_aligned() {
        let mut x = 44;
        let p = &raw mut x;
        assert_eq!(p.debug_ensure_aligned(), &raw mut x);
    }

    #[test]
    fn ptr() {
        // test: new  deref
        let x = 7u8;
        let px = unsafe { Ptr::<'_, Aligned>::new(NonNull::from(&x).cast()) };
        assert_eq!(unsafe { *px.deref::<u8>() }, x);

        // test: Ptr::from  as_ptr
        let y = 71;
        let addr = (&raw const y) as usize;
        let py = Ptr::from(&y);
        assert_eq!(py.as_ptr() as usize, addr);

        // test: byte_add  byte_offset
        let py = unsafe { py.byte_add(4).byte_offset(-4) };
        assert_eq!(unsafe { *py.deref::<i32>() }, y);

        // test: NonNull::from  into_mut
        let nn = NonNull::<u8>::from(py);
        let pm = unsafe { py.into_mut() };
        assert_eq!(nn.as_ptr(), pm.as_ptr());

        // test: Debug Display to_unaligned
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
    fn ptr_mut() {
        // test: new  deref_mut
        let mut x = 7u8;
        let px = unsafe { PtrMut::<'_, Aligned>::new(NonNull::from(&mut x).cast()) };
        assert_eq!(unsafe { *px.deref_mut::<u8>() }, x);

        // test: PtrMut::from  as_ptr
        let mut y = 71;
        let addr = (&raw const y) as usize;
        let py = PtrMut::from(&mut y);
        assert_eq!(py.as_ptr() as usize, addr);

        // test: promote NonNull::from  to_unaligned
        let op = unsafe { PtrMut::from(&mut y).promote() }.as_ptr();
        let nn = NonNull::<u8>::from(PtrMut::from(&mut y)).as_ptr();
        assert_eq!(op, nn);

        // test: byte_add  byte_offset
        let py = PtrMut::from(&mut y);
        let py = unsafe { py.byte_add(4).byte_offset(-4) };
        assert_eq!(unsafe { *py.deref_mut::<i32>() }, y);

        // test: to_unaligned Debug Display
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
    fn owning_ptr() {
        // test: new  read
        let mut x = 7u8;
        let px = unsafe { OwningPtr::<'_, Aligned>::new(NonNull::from(&mut x).cast()) };
        assert_eq!(unsafe { px.read::<u8>() }, x);

        // test: make
        let y: i32 = 71;
        OwningPtr::make(y, |py| {
            assert_eq!(unsafe { py.read::<i32>() }, 71);
        });

        // test: to_unaligned Display Debug as_ptr read_unaligned
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

        assert_eq!(unsafe { unaligned.read_unaligned::<u32>() }, z);
    }

    #[test]
    fn moving_ptr() {
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
            let _mp = unsafe { MovingPtr::from_value(&mut v) };
        }
        assert_eq!(counter.get(), 1);
    }

    #[test]
    fn thin_slice_ptr() {
        let mut arr = [0; 5];
        let mut counter = 0;
        for it in &mut arr {
            *it += counter;
            counter += 1;
        }

        // test: get
        let ptr: ThinSlicePtr<'_, i32> = arr.as_slice().into();
        for it in 0..5 {
            assert_eq!(unsafe { *ptr.get(it) }, it as i32);
        }

        // test: copy clone
        let p1 = ptr;
        let p2 = ptr.clone();
        for it in 0..5 {
            assert_eq!(unsafe { *p1.get(it) }, it as i32);
            assert_eq!(unsafe { *p1.get(it) }, unsafe { *p2.get(it) });
            assert_eq!(unsafe { *p1.get(it) }, unsafe { *ptr.get(it) });
        }
    }

    #[test]
    fn unsafe_cell_deref() {
        let t = UnsafeCell::new(123);
        // test: deref_mut
        let mut_ref = unsafe { t.deref_mut() };
        assert_eq!(*mut_ref, 123);
        *mut_ref += 5;
        // test: deref
        let immut_ref = unsafe { t.deref() };
        assert_eq!(*immut_ref, 128);

        // test: read
        assert_eq!(unsafe { t.read() }, 128);
    }
}
