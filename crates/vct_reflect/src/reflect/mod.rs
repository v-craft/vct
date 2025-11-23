use core::hash::BuildHasher;
use vct_utils::hash::{FixedHash, DefaultHasher};

#[inline(always)]
pub fn reflect_hasher() -> DefaultHasher<'static> {
    FixedHash.build_hasher()
}

mod reflect_impl;
pub use reflect_impl::Reflect;

mod partial_reflect;
pub use partial_reflect::PartialReflect;
