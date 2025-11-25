use core::any::{Any, TypeId};

/// Check if the Self type is the specified type `T`, compare using [`TypeId`]
pub trait Is {
    /// Checks if the current type "is" another type, using a [`TypeId`] equality comparison.
    /// This is most useful in the context of generic logic.
    fn is<T: Any>() -> bool;
}

impl<A: Any> Is for A {
    /// # Example
    /// 
    /// ```
    /// # use vct_reflect::Is;
    /// # use core::any::Any;
    /// 
    /// assert!(u32::is::<u32>());
    /// assert!(!usize::is::<u32>());
    /// ```
    #[inline(always)]
    fn is<T: Any>() -> bool {
        TypeId::of::<A>() == TypeId::of::<T>()
    }
}
