use core::{any::TypeId, hash::Hash};
use crate::{
    hash::{Hashed, NoOpHash},
    collections::{
        HashMap,
        hash_map::{Entry, RawEntryMut},
    },
};

/// A [`HashMap`] pre-configured to use [`Hashed`] keys and [`PassHash`] passthrough hashing.
/// Iteration order only depends on the order of insertions and deletions.
pub type PreHashMap<K, V> = HashMap<Hashed<K>, V, NoOpHash>;

impl<K, V> PreHashMap<K, V> {
    /// Create a empty [`PreHashMap`] 
    #[inline]
    pub const fn new() -> Self {
        Self::with_hasher(NoOpHash)
    }
}

impl<K: Hash + Eq + PartialEq + Clone, V> PreHashMap<K, V> {
    /// Tries to get or insert the value for the given `key` using the pre-computed hash first.
    /// If the [`PreHashMap`] does not already contain the `key`, it will clone it and insert
    /// the value returned by `func`.
    pub fn get_or_insert_with<F: FnOnce() -> V>(&mut self, key: &Hashed<K>, func: F) -> &mut V {
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

/// A specialized hashmap type with Key of [`TypeId`]
/// Iteration order only depends on the order of insertions and deletions.
pub type TypeIdMap<V> = HashMap<TypeId, V, NoOpHash>;

impl<V> TypeIdMap<V> {
    /// Create a empty [`TypeIdMap`]
    #[inline]
    pub const fn new() -> Self {
        Self::with_hasher(NoOpHash)
    }

    /// Inserts a value for the type `T`.
    #[inline]
    pub fn insert_type<T: ?Sized + 'static>(&mut self, v: V) -> Option<V> {
        self.insert(TypeId::of::<T>(), v)
    }

    /// Returns a reference to the value for type `T`, if one exists.
    #[inline]
    pub fn get_type<T: ?Sized + 'static>(&self) -> Option<&V> {
        self.get(&TypeId::of::<T>())
    }

    /// Returns a mutable reference to the value for type `T`, if one exists.
    #[inline]
    pub fn get_type_mut<T: ?Sized + 'static>(&mut self) -> Option<&mut V> {
        self.get_mut(&TypeId::of::<T>())
    }

    /// Removes type `T` from the map, returning the value for this
    /// key if it was previously present.
    #[inline]
    pub fn remove_type<T: ?Sized + 'static>(&mut self) -> Option<V> {
        self.remove(&TypeId::of::<T>())
    }

    /// Gets the type `T`'s entry in the map for in-place manipulation.
    #[inline]
    pub fn entry_type<T: ?Sized + 'static>(&mut self) -> Entry<'_, TypeId, V, NoOpHash> {
        self.entry(TypeId::of::<T>())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
