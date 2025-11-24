use core::fmt;
use alloc::{
    format,
    vec::Vec,
    boxed::Box,
};
use vct_utils::collections::{hash_table, HashTable};
use crate::{
    PartialReflect, Reflect,
    ops::{
        ReflectRef, ReflectMut, ReflectOwned, ApplyError,
    },
    info::{
        TypeInfo, TypePath, MaybeTyped, ReflectKind, MapInfo,
    },
};


#[derive(Default)]
pub struct DynamicMap {
    target_type: Option<&'static TypeInfo>,
    hash_table: HashTable<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)>,
}

impl TypePath for DynamicMap {
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicMap"
    }
    fn short_type_path() -> &'static str {
        "DynamicMap"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynamicMap")
    }
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicMap {
    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::Map(_)),
                "expected TypeInfo::Map but received: {target_type:?}"
            );
        }
        self.target_type = target_type;
    }

    #[inline]
    pub fn insert<K: PartialReflect, V: PartialReflect>(&mut self, key: K, value: V) {
        self.insert_boxed(Box::new(key), Box::new(value));
    }

    fn internal_hash(value: &dyn PartialReflect) -> u64 {
        value.reflect_hash().expect(&{
            let type_path = (value).reflect_type_path();
            if !value.is_dynamic() {
                format!("the given value of type `{}` does not support hashing", type_path)
            } else {
                match value.get_target_type_info() {
                    None => format!("the dynamic type `{}` does not support hashing", type_path),
                    Some(target) => format!(
                        "the dynamic type `{}` (target: `{}`) does not support hashing",
                        type_path,
                        target.type_path(),
                    ),
                }
            }
        })
    }

    fn internal_eq(
        key: &dyn PartialReflect,
    ) -> impl FnMut(&(Box<dyn PartialReflect>, Box<dyn PartialReflect>)) -> bool + '_ {
        |(other, _)| {
            key
            .reflect_partial_eq(&**other)
            .expect("underlying type does not reflect `PartialEq` and hence doesn't support equality checks")
        }
    }
}

impl PartialReflect for DynamicMap {
    #[inline]
    fn get_target_type_info(&self) -> Option<&'static TypeInfo> {
        self.target_type
    }

    #[inline]
    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    #[inline]
    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }

    #[inline]
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    #[inline]
    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        None
    }

    #[inline]
    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        None
    }

    #[inline]
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Err(self)
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        let other = value.reflect_ref().as_map()?;
        for (key, other_val) in other.iter() {
            if let Some(self_val) = self.get_mut(key) {
                self_val.try_apply(other_val)?;
            } else {
                self.insert_boxed(key.to_dynamic(), other_val.to_dynamic());
            }
        }
        self.retain(&mut |key, _| other.get(key).is_some());

        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Map
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Map(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Map(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Map(self)
    }

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        let ReflectRef::Map(other) = other.reflect_ref() else {
            return Some(false);
        };

        if self.len() != other.len() {
            return Some(false);
        }

        for (key, val) in self.iter() {
            if let Some(other_val) = other.get(key) {
                let result = val.reflect_partial_eq(other_val);
                if result != Some(true) {
                    return result;
                }
            } else {
                return Some(false);
            }
        }

        Some(true)
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicMap(")?;
        map_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicMap {}

impl fmt::Debug for DynamicMap {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl FromIterator<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)> for DynamicMap {
    fn from_iter<I: IntoIterator<Item = (Box<dyn PartialReflect>, Box<dyn PartialReflect>)>>(
        items: I,
    ) -> Self {
        // inline for compile-time runing
        let mut this = Self { 
            target_type: None,
            hash_table: HashTable::new() 
        };

        for (key, value) in items.into_iter() {
            this.insert_boxed(key, value);
        }
        this
    }
}

impl<K: Reflect, V: Reflect> FromIterator<(K, V)> for DynamicMap {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(items: I) -> Self {
        // inline for compile-time runing
        let mut this = Self { 
            target_type: None,
            hash_table: HashTable::new() 
        };

        for (key, value) in items.into_iter() {
            this.insert(key, value);
        }
        this
    }
}

impl IntoIterator for DynamicMap {
    type Item = (Box<dyn PartialReflect>, Box<dyn PartialReflect>);
    type IntoIter = hash_table::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicMap {
    type Item = (&'a dyn PartialReflect, &'a dyn PartialReflect);
    type IntoIter = core::iter::Map<
        hash_table::Iter<'a, (Box<dyn PartialReflect>, Box<dyn PartialReflect>)>,
        fn(&'a (Box<dyn PartialReflect>, Box<dyn PartialReflect>)) -> Self::Item,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
    }
}
pub trait Map: PartialReflect {
    fn get(&self, key: &dyn PartialReflect) -> Option<&dyn PartialReflect>;

    fn get_mut(&mut self, key: &dyn PartialReflect) -> Option<&mut dyn PartialReflect>;

    fn len(&self) -> usize;

    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn PartialReflect, &dyn PartialReflect)> + '_>;

    fn drain(&mut self) -> Vec<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)>;

    fn retain(&mut self, f: &mut dyn FnMut(&dyn PartialReflect, &mut dyn PartialReflect) -> bool);

    fn to_dynamic_map(&self) -> DynamicMap {
        let mut map = DynamicMap::default();
        map.set_target_type_info(self.get_target_type_info());
        for (key, value) in self.iter() {
            map.insert_boxed(key.to_dynamic(), value.to_dynamic());
        }
        map
    }

    fn insert_boxed(
        &mut self,
        key: Box<dyn PartialReflect>,
        value: Box<dyn PartialReflect>,
    ) -> Option<Box<dyn PartialReflect>>;

    fn remove(&mut self, key: &dyn PartialReflect) -> Option<Box<dyn PartialReflect>>;

    fn get_target_map_info(&self) -> Option<&'static MapInfo> {
        self.get_target_type_info()?.as_map().ok()
    }
}

impl Map for DynamicMap {
    #[inline]
    fn get(&self, key: &dyn PartialReflect) -> Option<&dyn PartialReflect> {
        self.hash_table
            .find(Self::internal_hash(key), Self::internal_eq(key))
            .map(|(_, value)| &**value)
    }

    #[inline]
    fn get_mut(&mut self, key: &dyn PartialReflect) -> Option<&mut dyn PartialReflect> {
        self.hash_table
            .find_mut(Self::internal_hash(key), Self::internal_eq(key))
            .map(|(_, value)| &mut **value)
    }

    #[inline]
    fn len(&self) -> usize {
        self.hash_table.len()
    }

    #[inline]
    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn PartialReflect, &dyn PartialReflect)> + '_> {
        let iter = self.hash_table.iter().map(|(k, v)| (&**k, &**v));
        Box::new(iter)
    }

    #[inline]
    fn drain(&mut self) -> Vec<(Box<dyn PartialReflect>, Box<dyn PartialReflect>)> {
        self.hash_table.drain().collect()
    }

    #[inline]
    fn retain(&mut self, f: &mut dyn FnMut(&dyn PartialReflect, &mut dyn PartialReflect) -> bool) {
        self.hash_table
            .retain(move |(key, value)| f(&**key, &mut **value));
    }

    fn insert_boxed(
        &mut self,
        key: Box<dyn PartialReflect>,
        value: Box<dyn PartialReflect>,
    ) -> Option<Box<dyn PartialReflect>> {
        assert_eq!(
            key.reflect_partial_eq(&*key),
            Some(true),
            "keys inserted in `Map`-like types are expected to reflect `PartialEq`"
        );

        let hash = Self::internal_hash(&*key);
        let eq = Self::internal_eq(&*key);
        match self.hash_table.find_mut(hash, eq) {
            Some((_, old)) => Some(core::mem::replace(old, value)),
            None => {
                self.hash_table.insert_unique(
                    Self::internal_hash(key.as_ref()),
                    (key, value),
                    |(key, _)| Self::internal_hash(&**key),
                );
                None
            }
        }
    }

    fn remove(&mut self, key: &dyn PartialReflect) -> Option<Box<dyn PartialReflect>> {
        let hash = Self::internal_hash(key);
        let eq = Self::internal_eq(key);
        match self.hash_table.find_entry(hash, eq) {
            Ok(entry) => {
                let ((_, old_value), _) = entry.remove();
                Some(old_value)
            }
            Err(_) => None,
        }
    }
}


pub fn map_partial_eq<M: Map + ?Sized>(x: &M, y: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::Map(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.len() != y.len() {
        return Some(false);
    }

    for (key, val) in x.iter() {
        if let Some(y_val) = y.get(key) {
            let result = val.reflect_partial_eq(y_val);
            if result != Some(true) {
                return result;
            }
        } else {
            return Some(false);
        }
    }

    Some(true)
}

pub fn map_debug(dyn_map: &dyn Map, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_map();
    for (key, value) in dyn_map.iter() {
        debug.entry(&key as &dyn fmt::Debug, &value as &dyn fmt::Debug);
    }
    debug.finish()
}

