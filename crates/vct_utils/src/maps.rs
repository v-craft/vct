//! 提供两个额外的 map 容器
//!
//! 此库仅在启用 alloc 时生效
use core::{any::TypeId, hash::Hash};

use crate::collections::{
    HashMap,
    hash_map::{Entry, RawEntryMut},
};
use crate::hash::{Hashed, NoOpHash};

/// 一个预计算了哈希值并使用 [`Hashed`] 作为键的 [`HashMap`]
///
/// 使用 [`NoOpHash`] 计算哈希值，即直接读取 [`Hashed`] 中存储的 `u64` 数据
pub type PreHashMap<K, V> = HashMap<Hashed<K>, V, NoOpHash>;

pub trait PreHashMapExt<K, V> {
    /// 如果元素存在则获取可变引用，不存在则先插入后获取
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V;
}

impl<K: Hash + Eq + PartialEq + Clone, V> PreHashMapExt<K, V> for PreHashMap<K, V> {
    // 此代码可能频繁调用，虽然代码量较多，依然建议内联
    #[inline]
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V {
        let entry: RawEntryMut<'_, Hashed<K>, V, NoOpHash> = self
            .raw_entry_mut()
            .from_key_hashed_nocheck(key.hash(), key);

        match entry {
            RawEntryMut::Occupied(entry) => entry.into_mut(),
            RawEntryMut::Vacant(entry) => {
                let (_, value) = entry.insert_hashed_nocheck(key.hash(), key.clone(), func());
                value
            }
        }
    }
}

/// 一个将 [`TypeId`] 作为键的 [`HashMap`]
pub type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;

impl<V> TypeIdMap<V> {
    /// 创建一个空的 [`TypeIdMap`] 。
    pub const fn new() -> Self {
        Self::with_hasher(NoOpHash)
    }
}

pub trait TypeIdMapExt<V> {
    fn insert_type<T: ?Sized + 'static>(&mut self, v: V) -> Option<V>;
    fn get_type<T: ?Sized + 'static>(&self) -> Option<&V>;
    fn get_type_mut<T: ?Sized + 'static>(&mut self) -> Option<&mut V>;
    fn remove_type<T: ?Sized + 'static>(&mut self) -> Option<V>;
    fn entry_type<T: ?Sized + 'static>(&mut self) -> Entry<'_, TypeId, V, NoOpHash>;
}

impl<V> TypeIdMapExt<V> for TypeIdMap<V> {
    #[inline]
    fn insert_type<T: ?Sized + 'static>(&mut self, v: V) -> Option<V> {
        self.insert(TypeId::of::<T>(), v)
    }

    #[inline]
    fn get_type<T: ?Sized + 'static>(&self) -> Option<&V> {
        self.get(&TypeId::of::<T>())
    }

    #[inline]
    fn get_type_mut<T: ?Sized + 'static>(&mut self) -> Option<&mut V> {
        self.get_mut(&TypeId::of::<T>())
    }

    #[inline]
    fn remove_type<T: ?Sized + 'static>(&mut self) -> Option<V> {
        self.remove(&TypeId::of::<T>())
    }

    #[inline]
    fn entry_type<T: ?Sized + 'static>(&mut self) -> Entry<'_, TypeId, V, NoOpHash> {
        self.entry(TypeId::of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typeid_hash_call() {
        // 确保 TypeId 的 hash 将调用 write_u64 而非 write
        struct Hasher;

        impl core::hash::Hasher for Hasher {
            fn finish(&self) -> u64 {
                0
            }
            fn write(&mut self, _: &[u8]) {
                panic!("Hashing of core::any::TypeId changed");
            }
            fn write_u64(&mut self, _: u64) {}
        }

        Hash::hash(&TypeId::of::<()>(), &mut Hasher);
    }

    #[test]
    fn typeid_map() {
        struct MyType;

        let mut map = TypeIdMap::default();
        map.insert(TypeId::of::<MyType>(), 7);
        assert_eq!(map.get(&TypeId::of::<MyType>()), Some(&7));

        let Some(val) = map.insert_type::<MyType>(8) else {
            panic!();
        };
        assert_eq!(val, 7);

        if let Some(val) = map.get_type_mut::<MyType>() {
            *val += 10;
        }
        assert_eq!(map.get_type::<MyType>(), Some(&18));

        assert_eq!(map.len(), 1usize);
        map.remove_type::<MyType>();
        assert_eq!(map.len(), 0usize);

        match map.entry_type::<MyType>() {
            Entry::Occupied(_) => {
                panic!("expected vacant entry");
            }
            Entry::Vacant(v) => {
                v.insert(123);
            }
        }
        match map.entry_type::<MyType>() {
            Entry::Occupied(o) => {
                assert_eq!(*o.get(), 123);
            }
            Entry::Vacant(_) => {
                panic!("expected occupied entry");
            }
        }
    }
}
