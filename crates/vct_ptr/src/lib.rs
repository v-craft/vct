#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]
#![expect(unsafe_code, reason = "Raw pointers are inherently unsafe.")]

use core::{
    marker::PhantomData,
    cell::UnsafeCell,
    num::NonZeroUsize,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
    mem::{self, ManuallyDrop, MaybeUninit},
    fmt::{self, Debug, Formatter, Pointer},
};

/// A newtype around [`NonNull<T>`] that only allows conversion to read-only borrows or pointers.
/// 
/// This type can be thought of as the `*const T` to [`NonNull<T>`]'s `*mut T`.
#[repr(transparent)]
pub struct ConstNonNull<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> ConstNonNull<T> {
    /// Creates a new `ConstNonNull` if `ptr` is non-null.
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
        // `map` is not stable const fn yet
        match NonNull::new(ptr.cast_mut()) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    /// Creates a new `ConstNonNull`, `ptr` must be non-null.
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
    /// *Incorrect* usage of this function:
    ///
    /// ```rust,no_run
    /// use vct_ptr::ConstNonNull;
    ///
    /// // NEVER DO THAT!!! This is undefined behavior. ⚠️
    /// let ptr = unsafe { ConstNonNull::<u32>::new_unchecked(core::ptr::null()) };
    /// ```
    #[inline]
    pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
        unsafe { Self(NonNull::new_unchecked(ptr.cast_mut())) }
    }

    /// Returns a shared reference to the value.
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that all of the following is true:
    ///
    /// - The pointer must be [properly aligned].
    /// 
    /// - It must be "dereferenceable" in the sense defined in [the module documentation].
    /// 
    /// - The pointer must point to an initialized instance of `T`.
    /// 
    /// - You must enforce Rust's aliasing rules, since the returned lifetime `'a` is
    ///   arbitrarily chosen and does not necessarily reflect the actual lifetime of the data.
    ///   In particular, while this reference exists, the memory the pointer points to must
    ///   not get mutated (except inside `UnsafeCell`).
    ///
    /// This applies even if the result of this method is unused!
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
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe  fn as_ref<'a>(&self) -> &'a T {
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

/// Used as a type argument to [`Ptr`], [`PtrMut`], [`ManualPtr`], and [`AutoPtr`] 
/// to specify that the pointer is guaranteed to be [aligned].
/// 
/// [aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Debug, Copy, Clone)]
pub struct Aligned;

/// Used as a type argument to [`Ptr`], [`PtrMut`], [`ManualPtr`], and [`AutoPtr`] to specify that the pointer may not [aligned].
/// 
/// [aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Debug, Copy, Clone)]
pub struct Unaligned;

mod seal_aligned {
    pub trait Sealed {}
    impl Sealed for super::Aligned {}
    impl Sealed for super::Unaligned {}
}

pub trait IsAligned: seal_aligned::Sealed {
    unsafe fn read_ptr<T>(ptr: *const T) -> T;
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize);
    unsafe fn drop_in_place<T>(ptr: *mut T);
}

/// Trait that is only implemented for [`Aligned`] and [`Unaligned`]
impl IsAligned for Aligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        // SAFETY:
        //  - The caller is required to ensure that `src` must be valid for reads.
        //  - The caller is required to ensure that `src` points to a valid instance of type `T`.
        //  - This type is `Aligned` so the caller must ensure that `src` is properly aligned for type `T`.
        unsafe { ptr.read() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        // SAFETY:
        //  - The caller is required to ensure that `src` must be valid for reads.
        //  - The caller is required to ensure that `dst` must be valid for writes.
        //  - The caller is required to ensure that `src` and `dst` are aligned.
        //  - The caller is required to ensure that the memory region covered by `src`
        //    and `dst`, fitting up to `count` elements do not overlap.
        unsafe { ptr::copy_nonoverlapping(src, dst, count); }
    }

    #[inline]
    unsafe fn drop_in_place<T>(ptr: *mut T) {
        // SAFETY:
        //  - The caller is required to ensure that `ptr` must be valid for reads and writes.
        //  - The caller is required to ensure that `ptr` points to a valid instance of type `T`.
        //  - This type is `Aligned` so the caller must ensure that `ptr` is properly aligned for type `T`.
        //  - The caller is required to ensure that `ptr` points must be valid for dropping.
        //  - The caller is required to ensure that the value `ptr` points must not be used after this function call.
        unsafe { ptr::drop_in_place(ptr); }
    }
}


impl IsAligned for Unaligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        // SAFETY:
        //  - The caller is required to ensure that `src` must be valid for reads.
        //  - The caller is required to ensure that `src` points to a valid instance of type `T`.
        unsafe { ptr.read_unaligned() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        // SAFETY:
        //  - The caller is required to ensure that `src` must be valid for reads.
        //  - The caller is required to ensure that `dst` must be valid for writes.
        //  - This is doing a byte-wise copy. `src` and `dst` are always guaranteed to be
        //    aligned.
        //  - The caller is required to ensure that the memory region covered by `src`
        //    and `dst`, fitting up to `count` elements do not overlap.
        unsafe {
            ptr::copy_nonoverlapping::<u8>(
                src.cast::<u8>(),
                dst.cast::<u8>(),
                count * size_of::<T>()
            );
        }
    }

    #[inline]
    unsafe fn drop_in_place<T>(ptr: *mut T) {
        // SAFETY:
        //  - The caller is required to ensure that `ptr` must be valid for reads and writes.
        //  - The caller is required to ensure that `ptr` points to a valid instance of type `T`.
        //  - This type is not `Aligned` so the caller does not need to ensure that `ptr` is properly aligned for type `T`.
        //  - The caller is required to ensure that `ptr` points must be valid for dropping.
        //  - The caller is required to ensure that the value `ptr` points must not be used after this function call.
        unsafe {
            drop(ptr.read_unaligned());
        }
    }
}

/// Type-erased borrow of some unknown type chosen when constructing this type.
/// 
/// It may be helpful to think of this type as similar to `&'a dyn Any` but without
/// the metadata and able to point to data that does not correspond to a Rust type.
///
/// This type tries to act "borrow-like" which means that:
/// - It should be considered immutable: its target must not be changed while this pointer is alive.
/// - It must always point to a valid value of whatever the pointee type is.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must always be [properly aligned] for the unknown pointee type.
///
/// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Ptr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a u8, A)>);

/// Type-erased mutable borrow of some unknown type chosen when constructing this type.
///
/// It may be helpful to think of this type as similar to `&'a mut dyn Any` but without
/// the metadata and able to point to data that does not correspond to a Rust type.
///
/// This type tries to act "borrow-like" which means that:
/// - Pointer is considered exclusive and mutable. It cannot be cloned as this would lead to
///   aliased mutability.
/// - It must always point to a valid value of whatever the pointee type is.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must always be [properly aligned] for the unknown pointee type.
///
/// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
#[repr(transparent)]
pub struct PtrMut<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// Type-erased [`Box`]-like pointer to some unknown type chosen when constructing this type.
///
/// This pointer is _not_ responsible for freeing the memory pointed to by this pointer
/// as it may be pointing to an element in a `Vec` or to a local in a function etc.
/// 
/// Conceptually represents ownership of whatever data is being pointed to and so is
/// responsible for calling its `Drop` impl.
/// 
/// It may be helpful to think of this type as similar to `&'a mut ManuallyDrop<dyn Any>` but
/// without the metadata and able to point to data that does not correspond to a Rust type.
///
/// This type tries to act "borrow-like" which means that:
/// - Pointer should be considered exclusive and mutable. It cannot be cloned as this would lead
///   to aliased mutability and potentially use after free bugs.
/// - It must always point to a valid value of whatever the pointee type is.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - If `A` is [`Aligned`], the pointer must always be [properly aligned] for the unknown pointee type.
/// 
/// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
#[repr(transparent)]
pub struct ManualPtr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

/// A [`Box`]-like pointer for moving a value to a new memory location without needing to pass by
/// value.
/// 
/// This pointer is _not_ responsible for freeing the memory pointed to by this pointer
/// as it may be pointing to an element in a `Vec` or to a local in a function etc.
/// 
/// Conceptually represents ownership of whatever data is being pointed to and will call its
/// [`Drop`] impl upon being dropped. 
/// 
/// A value can be deconstructed into its fields via [`deconstruct_auto_ptr`], see it's documentation
/// for an example on how to use it.
///
/// This type tries to act "borrow-like" which means that:
/// - Pointer should be considered exclusive and mutable. It cannot be cloned as this would lead
///   to aliased mutability and potentially use after free bugs.
/// - It must always point to a valid value of whatever the pointee type is.
/// - The lifetime `'a` accurately represents how long the pointer is valid for.
/// - It does not support pointer arithmetic in any way.
/// - If `A` is [`Aligned`], the pointer must always be [properly aligned] for the type `T`.
///
/// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
/// [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
#[repr(transparent)]
pub struct AutoPtr<'a, T, A: IsAligned = Aligned>(NonNull<T>, PhantomData<(&'a mut T, A)>);

trait DebugEnsureAligned {
    fn debug_ensure_aligned(self) -> Self;
}

// Disable this for miri runs as it already checks if pointer to reference casts are properly aligned.
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
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value of whatever the pointee type is.
    /// - If the `A` type parameter is [`Aligned`] then `inner` must be [properly aligned] for the pointee type.
    /// - `inner` must have correct provenance to allow reads of the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`Ptr`] will stay valid and nothing
    ///   can mutate the pointee while this [`Ptr`] is live except through an [`UnsafeCell`].
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets the underlying pointer, erasing the associated lifetime.
    ///
    /// If possible, it is strongly encouraged to use [`deref`](Self::deref) over this function,
    /// as it retains the lifetime.
    #[inline]
    pub const fn as_ptr(self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// Transforms this [`Ptr<T>`] into a `&T` with the same lifetime
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`Ptr`].
    /// - If the type parameter `A` is [`Unaligned`] then this pointer must be [properly aligned]
    ///   for the pointee type `T`.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn deref<T>(self) -> &'a T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { &*ptr }
    }

    /// Transforms this [`Ptr`] into an [`PtrMut`]
    ///
    /// # Safety
    /// * The data pointed to by this `Ptr` must be valid for writes.
    /// * There must be no active references (mutable or otherwise) to the data underlying this `Ptr`.
    /// * Another [`PtrMut`] for the same [`Ptr`] must not be created until the first is dropped.
    #[inline]
    pub const unsafe fn to_mutable(self) -> PtrMut<'a, A> {
        PtrMut(self.0, PhantomData)
    }
}

impl<'a, T: ?Sized> From<&'a T> for Ptr<'a> {
    #[inline]
    fn from(val: &'a T) -> Self {
        // SAFETY: The returned pointer has the same lifetime as the passed reference.
        // Access is immutable.
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a, A: IsAligned> PtrMut<'a, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value of whatever the pointee type is.
    /// - If the `A` type parameter is [`Aligned`] then `inner` must be [properly aligned] for the pointee type.
    /// - `inner` must have correct provenance to allow read and writes of the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`PtrMut`] will stay valid and nothing
    ///   else can read or mutate the pointee while this [`PtrMut`] is live.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets the underlying pointer, erasing the associated lifetime.
    ///
    /// If possible, it is strongly encouraged to use [`deref_mut`](Self::deref_mut) over
    /// this function, as it retains the lifetime.
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
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn deref_mut<T>(self) -> &'a mut T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        // SAFETY: The caller ensures the pointee is of type `T` and the pointer can be dereferenced.
        unsafe { &mut *ptr }
    }

    /// Gets an immutable reference from this mutable reference
    #[inline]
    pub const fn as_const(&self) -> Ptr<'_, A> {
        // SAFETY: The `PtrMut` type's guarantees about the validity of this pointer are a superset of `Ptr` s guarantees
        unsafe { Ptr::new(self.0) }
    }

    /// Gets a [`PtrMut`] from this with a smaller lifetime.
    #[inline]
    pub const fn reborrow(&mut self) -> PtrMut<'_, A> {
        // SAFETY: the ptrmut we're borrowing from is assumed to be valid
        unsafe { PtrMut::new(self.0) }
    }

    /// Transforms this [`PtrMut`] into an [`ManualPtr`]
    ///
    /// # Safety
    /// Must have right to drop or move out of [`PtrMut`].
    #[inline]
    pub const unsafe fn to_manual(self) -> ManualPtr<'a, A> {
        ManualPtr(self.0, PhantomData)
    }

}

impl<'a, T: ?Sized> From<&'a mut T> for PtrMut<'a> {
    #[inline]
    fn from(val: &'a mut T) -> Self {
        // SAFETY: The returned pointer has the same lifetime as the passed reference.
        // The reference is mutable, and thus will not alias.
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a> ManualPtr<'a> {
    /// This exists mostly to reduce compile times;
    /// code is only duplicated per type, rather than per type with function called.
    unsafe fn make_internal<T>(temp: &mut ManuallyDrop<T>) -> ManualPtr<'_> {
        // SAFETY: The constraints of `to_manual` are upheld by caller.
        unsafe { PtrMut::from(temp).to_manual() }
    }

    /// Consumes a value and creates an [`ManualPtr`] to it while ensuring a double drop does not happen.
    #[inline]
    pub fn make<T, F: FnOnce(ManualPtr<'_>) -> R, R>(val: T, f: F) -> R {
        let mut val = ManuallyDrop::new(val);
        // SAFETY: The value behind the pointer will not get dropped or observed later,
        // so it's safe to promote it to an owning pointer.
        f(unsafe { Self::make_internal(&mut val) })
    }
}

impl<'a> ManualPtr<'a, Unaligned> {
    /// Consumes the [`ManualPtr`] to obtain ownership of the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`ManualPtr`].
    #[inline]
    pub const unsafe fn read_unaligned<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>();
        // SAFETY: The caller ensure the pointee is of type `T` and uphold safety for `read_unaligned`.
        unsafe { ptr.read_unaligned() }
    }
}

impl<'a, A: IsAligned> ManualPtr<'a, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// # Safety
    /// - `inner` must point to valid value of whatever the pointee type is.
    /// - If the `A` type parameter is [`Aligned`] then `inner` must be [properly aligned] for the pointee type.
    /// - `inner` must have correct provenance to allow read and writes of the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`ManualPtr`] will stay valid and nothing
    ///   else can read or mutate the pointee while this [`ManualPtr`] is live.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    /// Gets the underlying pointer, erasing the associated lifetime.
    ///
    /// If possible, it is strongly encouraged to use the other more type-safe functions
    /// over this function.
    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    /// Consumes the [`ManualPtr`] to obtain ownership of the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`ManualPtr`].
    /// - If the type parameter `A` is [`Unaligned`] then this pointer must be [properly aligned]
    ///   for the pointee type `T`.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn read<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { ptr.read() }
    }

    /// Consumes the [`ManualPtr`] to drop the underlying data of type `T`.
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`ManualPtr`].
    /// - If the type parameter `A` is [`Unaligned`] then this pointer must be [properly aligned]
    ///   for the pointee type `T`.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn drop_as<T>(self) {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { ptr.drop_in_place(); }
    }

    /// Gets an immutable pointer from this owned pointer.
    #[inline]
    pub const fn as_const(&self) -> Ptr<'_, A> {
        // SAFETY: The `Owning` type's guarantees about the validity of this pointer are a superset of `Ptr` s guarantees
        unsafe { Ptr::new(self.0) }
    }

    /// Gets a mutable pointer from this owned pointer.
    #[inline]
    pub const fn as_mutable(&mut self) -> PtrMut<'_, A> {
        // SAFETY: The `Owning` type's guarantees about the validity of this pointer are a superset of `Ptr` s guarantees
        unsafe { PtrMut::new(self.0) }
    }


    /// Casts to a concrete type as a [`AutoPtr`].
    ///
    /// # Safety
    /// - `T` must be the erased pointee type for this [`ManualPtr`].
    #[inline]
    pub const unsafe fn to_auto<T>(self) -> AutoPtr<'a, T, A> {
        AutoPtr(self.0.cast::<T>(), PhantomData)
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
            /// As the pointer is type-erased, there is no size information available. The provided
            /// `count` parameter is in raw bytes.
            ///
            /// *See also: [`ptr::offset`][ptr_offset]*
            ///
            /// # Safety
            /// - The offset cannot make the existing ptr null, or take it out of bounds for its allocation.
            /// - If the `A` type parameter is [`Aligned`] then the offset must not make the resulting pointer
            ///   be unaligned for the pointee type.
            /// - The value pointed by the resulting pointer must outlive the lifetime of this pointer.
            ///
            /// [ptr_offset]: https://doc.rust-lang.org/std/primitive.pointer.html#method.offset
            #[inline]
            pub const unsafe fn byte_offset(self, count: isize) -> Self {
                Self(
                    unsafe { NonNull::new_unchecked(self.as_ptr().offset(count)) },
                    PhantomData,
                )
            }

            /// Calculates the offset from a pointer (convenience for `.offset(count as isize)`).
            /// As the pointer is type-erased, there is no size information available. The provided
            /// `count` parameter is in raw bytes.
            ///
            /// *See also: [`ptr::add`][ptr_add]*
            ///
            /// # Safety
            /// - The offset cannot make the existing ptr null, or take it out of bounds for its allocation.
            /// - If the `A` type parameter is [`Aligned`] then the offset must not make the resulting pointer
            ///   be unaligned for the pointee type.
            /// - The value pointed by the resulting pointer must outlive the lifetime of this pointer.
            ///
            /// [ptr_add]: https://doc.rust-lang.org/std/primitive.pointer.html#method.add
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
impl_ptr!(ManualPtr);

impl<'a, T> AutoPtr<'a, T, Aligned> {
    /// Removes the alignment requirement of this pointer
    #[inline]
    pub const fn to_unaligned(self) -> AutoPtr<'a, T, Unaligned> {
        let value = AutoPtr(self.0, PhantomData);
        mem::forget(self);
        value
    }

    /// Creates a [`AutoPtr`] from a provided value of type `T`.
    ///
    /// For a safer alternative, it is strongly advised to use [`move_as_ptr`] where possible.
    ///
    /// # Safety
    /// - `value` must store a properly initialized value of type `T`.
    /// - Once the returned [`AutoPtr`] has been used, `value` must be treated as
    ///   it were uninitialized unless it was explicitly leaked via [`core::mem::forget`].
    #[inline]
    pub unsafe fn from_value(value: &'a mut MaybeUninit<T>) -> Self {
        AutoPtr(NonNull::from(value).cast::<T>(), PhantomData)
    }
}

impl<'a, T, A: IsAligned> AutoPtr<'a, T, A> {
    /// Creates a new instance from a raw pointer.
    ///
    /// For a safer alternative, it is strongly advised to use [`move_as_ptr`] where possible.
    ///
    /// # Safety
    /// - `inner` must point to valid value of `T`.
    /// - If the `A` type parameter is [`Aligned`] then `inner` must be [properly aligned] for `T`.
    /// - `inner` must have correct provenance to allow read and writes of the pointee type.
    /// - The lifetime `'a` must be constrained such that this [`AutoPtr`] will stay valid and nothing
    ///   else can read or mutate the pointee while this [`AutoPtr`] is live.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub const unsafe fn new(inner: NonNull<T>) -> Self {
        Self(inner, PhantomData)
    }

    /// Partially consume some fields inside of `self`.
    ///
    /// The partially returned value is returned back pointing to [`MaybeUninit<T>`].
    ///
    /// While calling this function is safe, care must be taken with the returned `AutoPtr` as it
    /// points to a value that may no longer be completely valid.
    ///
    /// # Example
    ///
    /// ```
    /// use core::mem::{offset_of, MaybeUninit, forget};
    /// use vct_ptr::{AutoPtr, move_as_ptr};
    /// # struct FieldAType(usize);
    /// # struct FieldBType(usize);
    /// # struct FieldCType(usize);
    /// # fn insert<T>(_ptr: AutoPtr<'_, T>) {}
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
    /// // Converts `parent` into a `AutoPtr`
    /// move_as_ptr!(parent);
    ///
    /// // SAFETY:
    /// // - `field_a` and `field_b` are both unique.
    /// let (partial_parent, ()) = AutoPtr::partial_make(parent, |parent_ptr| unsafe {
    ///   vct_ptr::deconstruct_auto_ptr!({
    ///     let Parent { field_a, field_b, field_c } = parent_ptr;
    ///   });
    ///   
    ///   insert(field_a);
    ///   insert(field_b);
    ///   forget(field_c);
    /// });
    ///
    /// // Move the rest of fields out of the parent.
    /// // SAFETY:
    /// // - `field_c` is by itself unique and does not conflict with the previous accesses
    /// //   inside `partial_move`.
    /// unsafe {
    ///   vct_ptr::deconstruct_auto_ptr!({
    ///     let MaybeUninit::<Parent> { field_a: _, field_b: _, field_c } = partial_parent;
    ///   });
    ///
    ///   insert(field_c);
    /// }
    /// ```
    ///
    /// [`forget`]: core::mem::forget
    #[inline]
    pub fn partial_make<R>(
        self,
        f: impl FnOnce(AutoPtr<'_, T, A>) -> R,
    ) -> (AutoPtr<'a, MaybeUninit<T>, A>, R) {
        let partial_ptr = self.0;
        let ret = f(self);
        (
            AutoPtr(partial_ptr.cast::<MaybeUninit<T>>(), PhantomData),
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
    /// This does *not* drop the value stored at `dst` and it's the caller's responsibility
    /// to ensure that it's properly dropped.
    ///
    /// # Safety
    ///  - `dst` must be valid for writes.
    ///  - If the `A` type parameter is [`Aligned`] then `dst` must be [properly aligned] for `T`.
    ///
    /// [properly aligned]: https://doc.rust-lang.org/std/ptr/index.html#alignment
    #[inline]
    pub unsafe fn write_to(self, dst: *mut T) {
        let src = self.0.as_ptr();
        mem::forget(self);
        // SAFETY:
        //  - `src` must be valid for reads as this pointer is considered to own the value it points to.
        //  - The caller is required to ensure that `dst` must be valid for writes.
        //  - As `A` is `Aligned`, the caller is required to ensure that `dst` is aligned and `src` must
        //    be aligned by the type's invariants.
        unsafe { A::copy_nonoverlapping(src, dst, 1) };
    }

    /// Writes the value pointed to by this pointer into `dst`.
    ///
    /// The value previously stored at `dst` will be dropped.
    #[inline]
    pub fn assign_to(self, dst: &mut T) {
        // SAFETY:
        // - `dst` is a mutable borrow, it must point to a valid instance of `T`.
        // - `dst` is a mutable borrow, it must point to value that is valid for dropping.
        // - `dst` is a mutable borrow, it must not alias any other access.
        unsafe { ptr::drop_in_place(dst); }
        // SAFETY:
        // - `dst` is a mutable borrow, it must be valid for writes.
        // - `dst` is a mutable borrow, it must always be aligned.
        unsafe { self.write_to(dst); }
    }

    /// Creates a [`AutoPtr`] for a specific field within `self`.
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
    /// use vct_ptr::{AutoPtr, move_as_ptr};
    /// # struct FieldAType(usize);
    /// # struct FieldBType(usize);
    /// # struct FieldCType(usize);
    /// # fn insert<T>(_ptr: AutoPtr<'_, T>) {}
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
    /// // Converts `parent` into a `AutoPtr`.
    /// move_as_ptr!(parent);
    ///
    /// unsafe {
    ///    let field_a = parent.move_field(|ptr| &raw mut (*ptr).field_a);
    ///    let field_b = parent.move_field(|ptr| &raw mut (*ptr).field_b);
    ///    let field_c = parent.move_field(|ptr| &raw mut (*ptr).field_c);
    ///    // Each call to insert may panic! Ensure that `parent_ptr` cannot be dropped before
    ///    // calling them!
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
    pub unsafe fn move_field<U>(&self, f: impl Fn(*mut T) -> *mut U) -> AutoPtr<'a, U, A> {
        AutoPtr(
            unsafe { NonNull::new_unchecked(f(self.0.as_ptr())) },
            PhantomData,
        )
    }

}

impl<'a, T, A: IsAligned> AutoPtr<'a, MaybeUninit<T>, A> {
    /// Creates a [`AutoPtr`] for a specific field within `self`.
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
    ) -> AutoPtr<'a, MaybeUninit<U>, A> {
        let self_ptr = self.0.as_ptr().cast::<T>();
        // SAFETY:
        // - The caller must ensure that `U` is the correct type for the field at `byte_offset` and thus
        //   cannot be null.
        // - `MaybeUninit<T>` is `repr(transparent)` and thus must have the same memory layout as `T``
        let field_ptr = unsafe { NonNull::new_unchecked(f(self_ptr)) };
        AutoPtr(field_ptr.cast::<MaybeUninit<U>>(), PhantomData)
    }

    /// Creates a [`AutoPtr`] pointing to a valid instance of `T`.
    ///
    /// See also: [`MaybeUninit::assume_init`].
    ///
    /// # Safety
    /// It's up to the caller to ensure that the value pointed to by `self`
    /// is really in an initialized state. Calling this when the content is not yet
    /// fully initialized causes immediate undefined behavior.
    #[inline]
    pub unsafe fn assume_init(self) -> AutoPtr<'a, T, A> {
        let value = AutoPtr(self.0.cast::<T>(), PhantomData);
        mem::forget(self);
        value
    }
}

impl<T, A: IsAligned> Pointer for AutoPtr<'_, T, A> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Pointer::fmt(&self.0, f)
    }
}

impl<T> Debug for AutoPtr<'_, T, Aligned> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AutoPtr<Aligned>({:?})", self.0)
    }
}

impl<T> Debug for AutoPtr<'_, T, Unaligned> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AutoPtr<Unaligned>({:?})", self.0)
    }
}

impl<'a, T, A: IsAligned> From<AutoPtr<'a, T, A>> for ManualPtr<'a, A> {
    #[inline]
    fn from(value: AutoPtr<'a, T, A>) -> Self {
        // SAFETY:
        // - `value.0` must always point to valid value of type `T`.
        // - The type parameter `A` is mirrored from input to output, keeping the same alignment guarantees.
        // - `value.0` by construction must have correct provenance to allow read and writes of type `T`.
        // - The lifetime `'a` is mirrored from input to output, keeping the same lifetime guarantees.
        // - `ManualPtr` maintains the same aliasing invariants as `AutoPtr`.
        let ptr = unsafe { ManualPtr::new(value.0.cast::<u8>()) };
        mem::forget(value);
        ptr
    }
}

impl<'a, T> TryFrom<AutoPtr<'a, T, Unaligned>> for AutoPtr<'a, T, Aligned> {
    type Error = AutoPtr<'a, T, Unaligned>;
    #[inline]
    fn try_from(value: AutoPtr<'a, T, Unaligned>) -> Result<Self, Self::Error> {
        let ptr = value.0;
        if ptr.as_ptr().is_aligned() {
            mem::forget(value);
            Ok(AutoPtr(ptr, PhantomData))
        } else {
            Err(value)
        }
    }
}

impl<T> Deref for AutoPtr<'_, T, Aligned> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &Self::Target {
        let ptr = self.0.as_ptr().debug_ensure_aligned();
        // SAFETY: This type owns the value it points to and the generic type parameter is `A` so this pointer must be aligned.
        unsafe { &*ptr }
    }
}

impl<T> DerefMut for AutoPtr<'_, T, Aligned> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.0.as_ptr().debug_ensure_aligned();
        // SAFETY: This type owns the value it points to and the generic type parameter is `A` so this pointer must be aligned.
        unsafe { &mut *ptr }
    }
}

impl<T, A: IsAligned> Drop for AutoPtr<'_, T, A> {
    fn drop(&mut self) {
        unsafe { A::drop_in_place(self.0.as_ptr()) };
    }
}

/// Conceptually equivalent to `&'a [T]` but with length information cut out for performance reasons
#[cfg(debug_assertions)]
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<&'a [T]>,
}

/// Conceptually equivalent to `&'a [T]` but with length information cut out for performance reasons
#[cfg(not(debug_assertions))]
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> ThinSlicePtr<'a, T> {
    /// Indexes the slice without doing bounds checks
    ///
    /// # Safety
    /// `index` must be in-bounds.
    #[inline]
    pub const unsafe fn get(self, index: usize) -> &'a T {
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
            // SAFETY: a reference can never be null
            ptr: unsafe { NonNull::new_unchecked(ptr.debug_ensure_aligned()) },
            #[cfg(debug_assertions)]
            len: slice.len(),
            _marker: PhantomData,
        }
    }
}

/// Creates a dangling pointer with specified alignment.
/// See [`NonNull::dangling`].
pub const fn dangling_with_align(align: NonZeroUsize) -> NonNull<u8> {
    debug_assert!(align.is_power_of_two(), "Alignment must be power of two.");
    
    unsafe {
        NonNull::new_unchecked(
            ptr::null_mut::<u8>().wrapping_add(align.get())
        )
    }
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
    unsafe fn read(self) -> T where T: Copy;
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

/// Safely converts a owned value into a [`AutoPtr`] while minimizing the number of stack copies.
///
/// This cannot be used as expression and must be used as a statement. Internally this macro works via variable shadowing.
#[macro_export]
macro_rules! move_as_ptr {
    ($value: ident) => {
        let mut $value = core::mem::MaybeUninit::new($value);
        let $value = unsafe { $crate::AutoPtr::from_value(&mut $value) };
    };
}

/// Helper macro used by [`deconstruct_auto_ptr`] to extract
/// the pattern from `field: pattern` or `field` shorthand.
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

/// Deconstructs a [`AutoPtr`] into its individual fields.
///
/// This consumes the [`AutoPtr`] and hands out [`AutoPtr`] wrappers around
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
/// and the macro will deconstruct a `AutoPtr<MaybeUninit<ParentType>>`
/// into `AutoPtr<MaybeUninit<FieldType>>` values.
///
/// # Examples
///
/// ## Structs
///
/// ```
/// use core::mem::{offset_of, MaybeUninit};
/// use vct_ptr::{AutoPtr, move_as_ptr};
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
/// // Converts `parent` into a `AutoPtr`
/// move_as_ptr!(parent);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `AutoPtr` of the field's type.
/// vct_ptr::deconstruct_auto_ptr!({
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
/// use vct_ptr::{AutoPtr, move_as_ptr};
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
/// // Converts `parent` into a `AutoPtr`
/// move_as_ptr!(parent);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `AutoPtr` of the field's type.
/// vct_ptr::deconstruct_auto_ptr!({
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
/// use vct_ptr::{AutoPtr, move_as_ptr};
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
/// // Converts `parent` into a `AutoPtr`
/// move_as_ptr!(parent);
///
/// // The field names must match the name used in the type definition.
/// // Each one will be a `AutoPtr` of the field's type.
/// vct_ptr::deconstruct_auto_ptr!({
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
/// [`assign_to`]: AutoPtr::assign_to
#[macro_export]
macro_rules! deconstruct_auto_ptr {
    ({ let tuple { $($field_index:tt: $pattern:pat),* $(,)? } = $ptr:expr ;}) => {
        let mut ptr: $crate::AutoPtr<_, _> = $ptr;
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
        let mut ptr: $crate::AutoPtr<core::mem::MaybeUninit<_>, _> = $ptr;
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
        let mut ptr: $crate::AutoPtr<_, _> = $ptr;
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
        let mut ptr: $crate::AutoPtr<core::mem::MaybeUninit<_>, _> = $ptr;
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


