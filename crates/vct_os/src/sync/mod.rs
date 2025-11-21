pub use sync_impl::{
    Arc, Barrier, BarrierWaitResult, LazyLock, LockResult, Mutex, MutexGuard, Once, OnceLock,
    OnceState, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError, TryLockResult,
    Weak,
};

pub mod atomic {
    pub use super::atomic_impl::{
        AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicPtr, AtomicU8,
        AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
    };
    pub use core::sync::atomic::Ordering;
}

mod once_flag;
pub use crate::once;
pub use once_flag::OnceFlag;

crate::cfg::switch! {
    crate::cfg::std => {
        use std::sync as sync_impl;
        use core::sync::atomic as atomic_impl;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}
