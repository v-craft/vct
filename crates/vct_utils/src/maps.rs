//! 提供两个额外的 map 容器
//! 
//! 此库仅在启用 alloc 时生效
use core::{any::TypeId, hash::Hash};

use crate::hash::{Hashed, NoOpHash};
use crate::collections::{
    HashMap,
    hash_map::{Entry, RawEntryMut}, 
};

pub type PreHashMap<K, V> = HashMap<Hashed<K>, V, NoOpHash>;

pub trait PreHashMapExt<K, V> {
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V;
}

impl<K: Hash + Eq + PartialEq + Clone, V> PreHashMapExt<K, V> for PreHashMap<K, V> {
    fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V {
        let entry = self
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


pub type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;

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
}
