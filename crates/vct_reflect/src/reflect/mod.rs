use core::hash::BuildHasher;
use vct_utils::hash::{DefaultHasher, FixedHash};

/// Get Fixed Hasher
#[inline(always)]
pub fn reflect_hasher() -> DefaultHasher<'static> {
    FixedHash.build_hasher()
}

mod reflect_impl;
pub use reflect_impl::Reflect;
pub(crate) use reflect_impl::impl_cast_reflect_fn;

mod from_reflect;
pub use from_reflect::FromReflect;

mod reflectable;
pub use reflectable::Reflectable;
