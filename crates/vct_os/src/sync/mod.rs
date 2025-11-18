
pub use sync_impl::{
    Barrier, BarrierWaitResult,
    LazyLock,
    Mutex, MutexGuard,
    Once, OnceLock, OnceState,
    LockResult, PoisonError, TryLockResult, TryLockError,
    RwLock, RwLockReadGuard, RwLockWriteGuard,
    Arc, Weak,
};

pub mod atomic {
    pub use core::sync::atomic::Ordering;
    pub use super::atomic_impl::{
        AtomicI8, AtomicU8,
        AtomicI16, AtomicU16,
        AtomicI32, AtomicU32,
        AtomicI64, AtomicU64,
        AtomicIsize, AtomicUsize,
        AtomicBool, AtomicPtr,
    };
}

crate::cfg::switch! {
    crate::cfg::std => {
        use std::sync as sync_impl;
        use core::sync::atomic as atomic_impl;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}


