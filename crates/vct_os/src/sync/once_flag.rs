use crate::sync::atomic;

/// 一个 [`AtomicBool`] 的封装，一次性的标志位
///
/// # 例
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
    /// 创建新对象，初始是 `true`
    pub const fn new() -> Self {
        Self(atomic::AtomicBool::new(true))
    }

    /// 将自身置为 false，并返回旧值
    pub fn set(&self) -> bool {
        self.0.swap(false, atomic::Ordering::Relaxed)
    }
}

impl Default for OnceFlag {
    /// 默认值调用 new，初始为 true
    fn default() -> Self {
        Self::new()
    }
}

/// 一个简化宏，用于保证代码只会调用一次
///
/// # 例
///
/// ```
/// # use vct_os::sync::once;
///
/// let mut count = 0;
///
/// for _ in 0..5 {
///     // 注意内部传入 “表达式” 而非 “语句”
///     once!( count += 1 );
/// }
///
/// assert_eq!(count, 1);
/// ```
#[macro_export]
macro_rules! once {
    ($expression:expr) => {{
        static SHOULD_FIRE: $crate::sync::OnceFlag = $crate::sync::OnceFlag::new();
        if SHOULD_FIRE.set() {
            $expression;
        }
    }};
}
