#![no_std]

use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
    mem::{self, ManuallyDrop, MaybeUninit},
    fmt::{self, Debug, Formatter, Pointer},
    ops::{Deref, DerefMut},
    num::NonZeroUsize,
    cell::UnsafeCell,
};


#[derive(Debug, Copy, Clone)]
pub struct Aligned;

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

impl IsAligned for Aligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        unsafe { ptr.read() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
        unsafe { ptr::copy_nonoverlapping(src, dst, count); }
    }

    #[inline]
    unsafe fn drop_in_place<T>(ptr: *mut T) {
        unsafe { ptr::drop_in_place(ptr); }
    }
}


impl IsAligned for Unaligned {
    #[inline]
    unsafe fn read_ptr<T>(ptr: *const T) -> T {
        unsafe { ptr.read_unaligned() }
    }

    #[inline]
    unsafe fn copy_nonoverlapping<T>(src: *const T, dst: *mut T, count: usize) {
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
        unsafe {
            drop(ptr.read_unaligned());
        }
    }
}

#[repr(transparent)]
pub struct ConstNonNull<T: ?Sized>(NonNull<T>);

impl<T: ?Sized> ConstNonNull<T> {
    #[inline]
    pub const fn new(ptr: *const T) -> Option<Self> {
        // NonNull::new(ptr.cast_mut()).map(Self)
        // `map` is not stable const fn yet
        match NonNull::new(ptr.cast_mut()) {
            Some(x) => Some(Self(x)),
            None => None,
        }
    }

    #[inline]
    pub const unsafe fn new_unchecked(ptr: *const T) -> Self {
        unsafe { Self(NonNull::new_unchecked(ptr.cast_mut())) }
    }

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

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Ptr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a u8, A)>);

#[repr(transparent)]
pub struct PtrMut<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

#[repr(transparent)]
pub struct ManualPtr<'a, A: IsAligned = Aligned>(NonNull<u8>, PhantomData<(&'a mut u8, A)>);

#[repr(transparent)]
pub struct AutoPtr<'a, T, A: IsAligned = Aligned>(NonNull<T>, PhantomData<(&'a mut T, A)>);

trait DebugEnsureAligned {
    fn debug_ensure_aligned(self) -> Self;
}

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
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    pub const fn as_ptr(self) -> *mut u8 {
        self.0.as_ptr()
    }

    #[inline]
    pub unsafe fn deref<T>(self) -> &'a T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { &*ptr }
    }

    #[inline]
    pub const unsafe fn to_mutable(self) -> PtrMut<'a, A> {
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
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    #[inline]
    pub unsafe fn deref_mut<T>(self) -> &'a mut T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { &mut *ptr }
    }

    #[inline]
    pub const fn as_ref(&self) -> Ptr<'_, A> {
        unsafe { Ptr::new(self.0) }
    }

    #[inline]
    pub const fn reborrow(&mut self) -> PtrMut<'_, A> {
        unsafe { PtrMut::new(self.0) }
    }

    #[inline]
    pub const unsafe fn to_manual(self) -> ManualPtr<'a, A> {
        ManualPtr(self.0, PhantomData)
    }

}

impl<'a, T: ?Sized> From<&'a mut T> for PtrMut<'a> {
    #[inline]
    fn from(val: &'a mut T) -> Self {
        unsafe { Self::new(NonNull::from(val).cast()) }
    }
}

impl<'a> ManualPtr<'a> {
    #[inline]
    pub fn make<T, F: FnOnce(ManualPtr<'_>) -> R, R>(val: T, f: F) -> R {
        let mut val = ManuallyDrop::new(val);
        f(unsafe { PtrMut::from(&mut val).to_manual() })
    }
}

impl<'a> ManualPtr<'a, Unaligned> {
    #[inline]
    pub const unsafe fn read_unaligned<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>();
        unsafe { ptr.read_unaligned() }
    }
}

impl<'a, A: IsAligned> ManualPtr<'a, A> {
    #[inline]
    pub const unsafe fn new(inner: NonNull<u8>) -> Self {
        Self(inner, PhantomData)
    }

    #[inline]
    pub const fn as_ptr(&self) -> *mut u8 {
        self.0.as_ptr()
    }

    #[inline]
    pub unsafe fn read<T>(self) -> T {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { ptr.read() }
    }

    #[inline]
    pub unsafe fn drop_as<T>(self) {
        let ptr = self.as_ptr().cast::<T>().debug_ensure_aligned();
        unsafe { ptr.drop_in_place(); }
    }

    #[inline]
    pub const fn as_ref(&self) -> Ptr<'_, A> {
        unsafe { Ptr::new(self.0) }
    }

    #[inline]
    pub const fn as_mut(&mut self) -> PtrMut<'_, A> {
        unsafe { PtrMut::new(self.0) }
    }

    #[inline]
    pub const unsafe fn to_auto<T>(self) -> AutoPtr<'a, T, A> {
        AutoPtr(self.0.cast::<T>(), PhantomData)
    } 
}

macro_rules! impl_ptr {
    ($ptr:ident) => {
        impl<'a> $ptr<'a, Aligned> {
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
            #[inline]
            pub const unsafe fn byte_offset(self, count: isize) -> Self {
                Self(
                    unsafe { NonNull::new_unchecked(self.as_ptr().offset(count)) },
                    PhantomData,
                )
            }

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
    #[inline]
    pub const fn to_unaligned(self) -> AutoPtr<'a, T, Unaligned> {
        let value = AutoPtr(self.0, PhantomData);
        mem::forget(self);
        value
    }

    #[inline]
    pub unsafe fn from_value(value: &'a mut MaybeUninit<T>) -> Self {
        AutoPtr(NonNull::from(value).cast::<T>(), PhantomData)
    }
}

impl<'a, T, A: IsAligned> AutoPtr<'a, T, A> {
    #[inline]
    pub const unsafe fn new(inner: NonNull<T>) -> Self {
        Self(inner, PhantomData)
    }

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

    #[inline]
    pub fn read(self) -> T {
        let value = unsafe { A::read_ptr(self.0.as_ptr()) };
        mem::forget(self);
        value
    }

    #[inline]
    pub unsafe fn write_to(self, dst: *mut T) {
        let src = self.0.as_ptr();
        mem::forget(self);
        unsafe { A::copy_nonoverlapping(src, dst, 1) };
    }

    #[inline]
    pub fn assign_to(self, dst: &mut T) {
        unsafe { ptr::drop_in_place(dst); }
        unsafe { self.write_to(dst); }
    }

    #[inline(always)]
    pub unsafe fn move_field<U>(&self, f: impl Fn(*mut T) -> *mut U) -> AutoPtr<'a, U, A> {
        AutoPtr(
            unsafe { NonNull::new_unchecked(f(self.0.as_ptr())) },
            PhantomData,
        )
    }

}

impl<'a, T, A: IsAligned> AutoPtr<'a, MaybeUninit<T>, A> {
    #[inline(always)]
    pub unsafe fn move_maybe_uninit_field<U>(
        &self,
        f: impl Fn(*mut T) -> *mut U,
    ) -> AutoPtr<'a, MaybeUninit<U>, A> {
        let self_ptr = self.0.as_ptr().cast::<T>();
        let field_ptr = unsafe { NonNull::new_unchecked(f(self_ptr)) };
        AutoPtr(field_ptr.cast::<MaybeUninit<U>>(), PhantomData)
    }

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
        unsafe { &*ptr }
    }
}

impl<T> DerefMut for AutoPtr<'_, T, Aligned> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        let ptr = self.0.as_ptr().debug_ensure_aligned();
        unsafe { &mut *ptr }
    }
}

impl<T, A: IsAligned> Drop for AutoPtr<'_, T, A> {
    fn drop(&mut self) {
        unsafe { A::drop_in_place(self.0.as_ptr()) };
    }
}

#[cfg(debug_assertions)]
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    _marker: PhantomData<&'a [T]>,
}

#[cfg(not(debug_assertions))]
#[derive(Copy, Clone)]
pub struct ThinSlicePtr<'a, T> {
    ptr: NonNull<T>,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> ThinSlicePtr<'a, T> {
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
            // SAFETY: a reference can never be null
            ptr: unsafe { NonNull::new_unchecked(ptr.debug_ensure_aligned()) },
            #[cfg(debug_assertions)]
            len: slice.len(),
            _marker: PhantomData,
        }
    }
}

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

pub trait UnsafeCellDeref<'a, T>: seal_unsafe_cell::Sealed {
    unsafe fn deref_mut(self) -> &'a mut T;
    unsafe fn deref(self) -> &'a T;
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

#[macro_export]
macro_rules! into_auto_ptr {
    ($value: ident) => {
        let mut $value = core::mem::MaybeUninit::new($value);
        let $value = unsafe { $crate::AutoPtr::from_value(&mut $value) };
    };
}

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

