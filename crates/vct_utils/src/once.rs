use core::default::Default;
use vct_os::sync::atomic::{Ordering, AtomicBool};

pub struct OnceFlag(AtomicBool);

impl OnceFlag {
    pub const fn new() -> Self {
        Self(AtomicBool::new(true))
    }

    pub fn set(&self) -> bool {
        self.0.swap(false, Ordering::Relaxed)
    }
}

impl Default for OnceFlag {
    fn default() -> Self {
        Self::new()
    }
}

#[macro_export]
macro_rules! once {
    ($expression:expr) => {{
        static SHOULD_FIRE: $crate::OnceFlag = $crate::OnceFlag::new();
        if SHOULD_FIRE.set() {
            $expression;
        }
    }};
}

