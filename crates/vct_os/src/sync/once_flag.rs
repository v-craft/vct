use crate::sync::atomic;

/// Wrapper around an [`AtomicBool`]
///
/// # Example
///
/// ```
/// # use vct_os::sync::OnceFlag;
///
/// let flag = OnceFlag::new();
/// let mut count = 0;
/// for _ in 0..5 {
///     if flag.set() {
///         count += 1;
///     }
/// }
/// assert_eq!(count, 1);
/// # // test
/// # let flag = OnceFlag::default();
/// # for _ in 0..5 {
/// #     if flag.set() {
/// #         count += 1;
/// #     }
/// # }
/// # assert_eq!(count, 2);
/// ```
pub struct OnceFlag(atomic::AtomicBool);

impl OnceFlag {
    /// Create new object, default inner value is `true`.
    #[inline]
    pub const fn new() -> Self {
        Self(atomic::AtomicBool::new(true))
    }

    /// Set inner value to `false` and return old value.
    #[inline]
    pub fn set(&self) -> bool {
        self.0.swap(false, atomic::Ordering::Relaxed)
    }
}

impl Default for OnceFlag {
    /// Call `new`, default inner value is `true`.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Call some expression only once per call site.
///
/// # Example
///
/// ```
/// # use vct_os::sync::once;
///
/// let mut count = 0;
///
/// for _ in 0..5 {
///     // use `expression` instead of `statement`
///     once!( count += 1 );
/// }
///
/// assert_eq!(count, 1);
/// ```
///
/// Impl through [`OnceFlag`] instead of [`Once`].
#[macro_export]
macro_rules! once {
    ($expression:expr) => {{
        static SHOULD_FIRE: $crate::sync::OnceFlag = $crate::sync::OnceFlag::new();
        if SHOULD_FIRE.set() {
            $expression;
        }
    }};
}
