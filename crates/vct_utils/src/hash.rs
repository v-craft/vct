use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher},
    marker::PhantomData,
    ops::Deref,
};

pub use foldhash::fast::{FixedState, FoldHasher as DefaultHasher, RandomState};

/// 一个随机生成的固定哈希种子
const FIXED_HASH: FixedState = FixedState::with_seed(0x95EE04C40326B271);

/// 固定哈希生成器
#[derive(Copy, Clone, Default, Debug)]
pub struct FixedHash;

impl BuildHasher for FixedHash {
    type Hasher = DefaultHasher<'static>;

    #[inline(always)]
    fn build_hasher(&self) -> Self::Hasher {
        FIXED_HASH.build_hasher()
    }
}

/// 用于存储已生成哈希值的对象
pub struct Hashed<V, S = FixedHash> {
    hash: u64,
    value: V,
    marker: PhantomData<S>,
}

impl<V: Hash, H: BuildHasher + Default> Hashed<V, H> {
    /// 预计算哈希值
    pub fn new(value: V) -> Self {
        Self {
            hash: H::default().hash_one(&value),
            value,
            marker: PhantomData,
        }
    }

    /// 获取预计算的指针
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

/// 一个不带操作的 Hasher 实现，仅使用 write_u64
#[derive(Debug, Default)]
pub struct NoOpHasher {
    hash: u64,
}

impl Hasher for NoOpHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }

    fn write(&mut self, bytes: &[u8]) {
        // 通常不建议使用此 write ，自定义场景请直接调用 write_64
        // 已确定 TypeId 会调用 write_u64 而非此函数
        self.hash = bytes.iter().fold(self.hash, |hash, b| {
            hash.rotate_left(8).wrapping_add(*b as u64)
        });
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

    fn build_hasher(&self) -> Self::Hasher {
        NoOpHasher::default()
    }
}
