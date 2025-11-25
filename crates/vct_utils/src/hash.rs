//! Provides replacements for `std::hash` items using [`foldhash`].
//!
//! Also provides some additional items beyond the standard library.

use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

pub use foldhash::fast::{FixedState, FoldHasher as DefaultHasher, RandomState};

/// A fixed hash seed(randomly generated)
const FIXED_HASH: FixedState = FixedState::with_seed(0x95EE04C40326B271);

/// Deterministic hasher based upon a random but fixed state.
#[derive(Copy, Clone, Default, Debug)]
pub struct FixedHash;

impl BuildHasher for FixedHash {
    type Hasher = DefaultHasher<'static>;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        FIXED_HASH.build_hasher()
    }
}

/// A pre-hashed value of a specific type.
/// Pre-hashing enables memoization of hashes that are expensive to compute.
///
/// It also enables faster [`PartialEq`] comparisons by short circuiting on hash equality.
/// See `PreHashMap` for a hashmap pre-configured to use [`Hashed`] keys.
pub struct Hashed<V, S = FixedHash> {
    hash: u64,
    value: V,
    marker: PhantomData<S>,
}

impl<V: Hash, H: BuildHasher + Default> Hashed<V, H> {
    /// Pre-hashes the given value using the [`BuildHasher`] configured in the [`Hashed`] type.
    #[inline]
    pub fn new(value: V) -> Self {
        Self {
            hash: H::default().hash_one(&value),
            value,
            marker: PhantomData,
        }
    }

    /// The pre-computed hash.
    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl<V, H> Hash for Hashed<V, H> {
    #[inline]
    fn hash<R: Hasher>(&self, state: &mut R) {
        state.write_u64(self.hash);
    }
}

impl<V, H> Deref for Hashed<V, H> {
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<V: PartialEq, H> PartialEq for Hashed<V, H> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.value.eq(&other.value)
    }
}

impl<V: Debug, H> Debug for Hashed<V, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Not inline: Debug allows for slight performance loss
        f.debug_struct("Hashed")
            .field("hash", &self.hash)
            .field("value", &self.value)
            .finish()
    }
}

impl<V: Clone, H> Clone for Hashed<V, H> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            hash: self.hash,
            value: self.value.clone(),
            marker: PhantomData,
        }
    }
}

impl<V: Copy, H> Copy for Hashed<V, H> {}

impl<V: Eq, H> Eq for Hashed<V, H> {}

/// A no-op hash that only works on `u64`s.
/// 
#[derive(Debug, Default)]
pub struct NoOpHasher {
    hash: u64,
}

impl Hasher for NoOpHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, _bytes: &[u8]) {
        // TypeId will call `write_u64` instead of this function
        panic!("NoOpHasher::write() should not be called; use write_u64 instead");
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.hash = i;
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct NoOpHash;

impl BuildHasher for NoOpHash {
    type Hasher = NoOpHasher;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        // manually inline
        NoOpHasher{ hash: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::any::TypeId;
    use alloc::{format, string::String};

    #[test]
    fn typeid_hash_call() {
        // Ensure that the hash of `TypeId` will call `write_u64` instead of `write`
        struct  MyHasher(u64);

        impl Hasher for MyHasher {
            fn finish(&self) -> u64 {
                0
            }
            fn write(&mut self, _: &[u8]) {
                panic!("Hashing of core::any::TypeId changed");
            }
            fn write_u64(&mut self, _: u64) {
                self.0 += 1;
            }
        }

        let mut hasher = MyHasher(0);

        Hash::hash(&TypeId::of::<()>(), &mut hasher);
        assert_eq!(hasher.0, 1u64);
    }

    #[test]
    fn no_op_hash() {
        let mut h0 = NoOpHasher::default();
        let h1 = NoOpHash::build_hasher(&NoOpHash);
        let h2 = NoOpHasher{ hash: 0 };
        assert_eq!(h0.hash, h1.hash);
        assert_eq!(h1.hash, h2.hash);

        37u64.hash(&mut h0);
        assert_eq!(h0.hash, 37u64);
        12u64.hash(&mut h0);
        assert_eq!(h0.hash, 12u64);
    }

    #[test]
    fn hashed() {
        // test: hash()
        let h = Hashed::<u64, NoOpHash> {
            hash: 0xDEADBEEFu64,
            value: 0u64,
            marker: PhantomData,
        };
        let mut hasher = NoOpHasher::default();
        Hash::hash(&h, &mut hasher);
        assert_eq!(hasher.finish(), 0xDEADBEEF);
        assert_eq!(h.hash(), 0xDEADBEEF);

        // test: new deref fmt
        let value = String::from("hello");
        let h1 = Hashed::<String>::new(value.clone());
        let h2 = h1.clone();
        // clone preserves hash and value
        assert_eq!(h1.hash(), h2.hash());
        assert_eq!(&*h1, &value);
        // Debug contains the type name "Hashed"
        assert!(format!("{:?}", h1).contains("Hashed"));

        // test: copy eq
        let a = Hashed {
            hash: 1u64,
            value: 10u32,
            marker: PhantomData::<FixedHash>,
        };
        let b = Hashed {
            hash: 1u64,
            value: 20u32,
            marker: PhantomData::<FixedHash>,
        };
        let a2 = a;
        assert_ne!(a, b);
        assert_eq!(a, a2);
    }

}
