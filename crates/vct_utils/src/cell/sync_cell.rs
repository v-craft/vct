#![expect(unsafe_code, reason = "SyncCell requires unsafe code.")]

//! A reimplementation of the currently unstable [`std::sync::Exclusive`]
//!
//! [`std::sync::Exclusive`]: https://doc.rust-lang.org/nightly/std/sync/struct.Exclusive.html

use core::ptr;

/// See [`Exclusive`](https://github.com/rust-lang/rust/issues/98407) for stdlib's upcoming implementation,
/// which should replace this one entirely.
///
/// # Example
///
/// ```
/// # use core::cell::Cell;
/// # use vct_utils::cell::SyncCell;
/// async fn other() {}
/// fn assert_sync<T: Sync>(t: T) {}
/// struct State<F> {
///     future: SyncCell<F>
/// }
///
/// assert_sync(State {
///     future: SyncCell::new(async {
///         // including Cell, but SyncCell is `sync`
///         let cell = Cell::new(1);
///         let cell_ref = &cell;
///         let val = cell_ref.get();
///     })
/// });
/// ```
#[repr(transparent)]
pub struct SyncCell<T: ?Sized> {
    inner: T,
}

// SAFETY: `Sync` only allows multithreaded access via immutable reference.
// 
// As `SyncCell` requires an exclusive reference to access the wrapped value for `!Sync` types,
// marking this type as `Sync` does not actually allow unsynchronized access to the inner value.
unsafe impl<T: ?Sized> Sync for SyncCell<T> {}

impl<T: Sized> SyncCell<T> {
    /// Create a new instance of a `SyncCell` from the given value.
    #[inline]
    pub const fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Deconstruct this `SyncCell` into its inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: ?Sized> SyncCell<T> {
    /// Get a mut reference to this `SyncCell`'s inner value.
    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// For types that implement [`Sync`], get shared access to this `SyncCell`'s inner value.
    #[inline]
    pub const fn as_ref(&self) -> &T
    where
        T: Sync,
    {
        &self.inner
    }

    /// Build a mutable reference to a `SyncCell` from a mutable reference
    /// to its inner value, to skip constructing with [`new()`](SyncCell::new()).
    #[inline]
    pub const fn from_mut(r: &mut T) -> &mut SyncCell<T> {
        let ptr = ptr::from_mut(r) as *mut SyncCell<T>;
        // SAFETY: repr is transparent, so refs have the same layout; 
        // and `SyncCell` properties are `&mut`-agnostic
        unsafe { &mut *ptr }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_cell() {
        let mut x = 5;
        let sc = SyncCell::from_mut(&mut x);
        *sc.get_mut() += 10;
        assert_eq!(*sc.as_ref(), 15);

        let mut sc = SyncCell::new(7);
        *sc.get_mut() += 10;
        assert_eq!(sc.into_inner(), 17);
    }
}
