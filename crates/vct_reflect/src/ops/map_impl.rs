use crate::{
    Reflect,
    cell::NonGenericTypeInfoCell,
    info::{MapInfo, ReflectKind, TypeInfo, TypePath, Typed, OpaqueInfo},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
    reflect::impl_cast_reflect_fn,
};
use alloc::{boxed::Box, format, vec::Vec};
use core::fmt;
use vct_utils::collections::{HashTable, hash_table};

/// Representing [`Map`], used to dynamically modify the type of data and information.
/// 
/// Dynamic types are special in that their TypeInfo is [`OpaqueInfo`], 
/// but other APIs are consistent with the type they represent, such as [`reflect_kind`], [`reflect_ref`]
/// 
/// [`reflect_kind`]: crate::Reflect::reflect_kind
/// [`reflect_ref`]: crate::Reflect::reflect_ref
#[derive(Default)]
pub struct DynamicMap {
    map_info: Option<&'static TypeInfo>,
    hash_table: HashTable<(Box<dyn Reflect>, Box<dyn Reflect>)>,
}

impl TypePath for DynamicMap {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicMap"
    }
    #[inline]
    fn type_name() -> &'static str {
        "DynamicMap"
    }
    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicMap")
    }
    #[inline]
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    #[inline]
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl Typed for DynamicMap {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl DynamicMap {
    #[inline]
    pub const fn new() -> DynamicMap {
        Self { map_info: None, hash_table: HashTable::new() }
    }

    /// Sets the [`TypeInfo`] to be represented by this `DynamicMap`.
    /// 
    /// # Panic
    /// 
    /// If the input is not list info or None.
    #[inline]
    pub fn set_type_info(&mut self, map_info: Option<&'static TypeInfo>) {
        match map_info {
            Some(TypeInfo::Map(_)) | None => {},
            _ => { panic!("Call `DynamicMap::set_type_info`, but the input is not map information or None.") },
        }

        self.map_info = map_info;
    }

    /// Inserts a typed key-value pair into the map.
    #[inline]
    pub fn insert<K: Reflect, V: Reflect>(&mut self, key: K, value: V) {
        self.insert_boxed(Box::new(key), Box::new(value));
    }

    fn internal_hash(value: &dyn Reflect) -> u64 {
        value.reflect_hash().expect(&{
            let type_path = (value).reflect_type_path();
            if !value.is_dynamic() {
                format!(
                    "the given value of type `{}` does not support hashing",
                    type_path
                )
            } else {
                match value.represented_type_info() {
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
        key: &dyn Reflect,
    ) -> impl FnMut(&(Box<dyn Reflect>, Box<dyn Reflect>)) -> bool + '_ {
        |(other, _)| {
            key
            .reflect_partial_eq(&**other)
            .expect("underlying type does not reflect `PartialEq` and hence doesn't support equality checks")
        }
    }
}

impl Reflect for DynamicMap {
    impl_cast_reflect_fn!();

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.map_info
    }

    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        map_try_apply(self, value)
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

    #[inline]
    fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
        map_partial_eq(self, other)
    }

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicMap(")?;
        map_debug(self, f)?;
        write!(f, ")")
    }

}


impl fmt::Debug for DynamicMap {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl FromIterator<(Box<dyn Reflect>, Box<dyn Reflect>)> for DynamicMap {
    fn from_iter<I: IntoIterator<Item = (Box<dyn Reflect>, Box<dyn Reflect>)>>(
        items: I,
    ) -> Self {
        let mut this = DynamicMap::new();
        for (key, value) in items.into_iter() {
            this.insert_boxed(key, value);
        }
        this
    }
}

impl<K: Reflect, V: Reflect> FromIterator<(K, V)> for DynamicMap {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(items: I) -> Self {
        let mut this = DynamicMap::new();
        for (key, value) in items.into_iter() {
            this.insert(key, value);
        }
        this
    }
}

impl IntoIterator for DynamicMap {
    type Item = (Box<dyn Reflect>, Box<dyn Reflect>);
    type IntoIter = hash_table::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicMap {
    type Item = (&'a dyn Reflect, &'a dyn Reflect);
    type IntoIter = core::iter::Map<
        hash_table::Iter<'a, (Box<dyn Reflect>, Box<dyn Reflect>)>,
        fn(&'a (Box<dyn Reflect>, Box<dyn Reflect>)) -> Self::Item,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
    }
}

/// A trait used to power [map-like] operations via [reflection].
///
/// Maps contain zero or more entries of a key and its associated value,
/// and correspond to types like `HashMap` and `BTreeMap`.
/// The order of these entries is not guaranteed by this trait.
pub trait Map: Reflect {
    /// Returns a reference to the value associated with the given key.
    ///
    /// If no value is associated with `key`, returns `None`.
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect>;

    /// Returns a mutable reference to the value associated with the given key.
    ///
    /// If no value is associated with `key`, returns `None`.
    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect>;

    /// Returns the number of elements in the map.
    fn len(&self) -> usize;

    /// Returns `true` if the list contains no elements.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the key-value pairs of the map.
    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn Reflect, &dyn Reflect)> + '_>;

    /// Drain the key-value pairs of this map to get a vector of owned values.
    ///
    /// After calling this function, `self` will be empty.
    fn drain(&mut self) -> Vec<(Box<dyn Reflect>, Box<dyn Reflect>)>;

    /// Retain only the elements specified by the predicate.
    ///
    /// In other words, remove all pairs `(k, v)` such that `f(&k, &mut v)` returns `false`.
    fn retain(&mut self, f: &mut dyn FnMut(&dyn Reflect, &mut dyn Reflect) -> bool);

    /// Creates a new [`DynamicMap`] from this map.
    fn to_dynamic_map(&self) -> DynamicMap {
        let mut map = DynamicMap::default();
        map.set_type_info(self.represented_type_info());
        for (key, value) in self.iter() {
            map.insert_boxed(key.to_dynamic(), value.to_dynamic());
        }
        map
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the value is updated, and the old value is returned.
    fn insert_boxed(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Option<Box<dyn Reflect>>;

    /// Removes an entry from the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    /// If the map did have this key present, the removed value is returned.
    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>>;

    /// Get actual [`MapInfo`] of underlying types.
    /// 
    /// If it is a dynamic type, it will return `None`.
    /// 
    /// If it is not a dynamic type and the returned value is not `None` or `MapInfo`, it will panic.
    /// (If you want to implement dynamic types yourself, please return None.)
    #[inline]
    fn reflect_map_info(&self) -> Option<&'static MapInfo> {
        self.reflect_type_info().as_map().ok()
    }

    /// Get the [`MapInfo`] of representation.
    /// 
    /// Normal types return their own information,
    /// while dynamic types return `None`` if they do not represent an object
    #[inline]
    fn represented_map_info(&self) -> Option<&'static MapInfo> {
        self.represented_type_info()?.as_map().ok()
    }
}

impl Map for DynamicMap {
    #[inline]
    fn get(&self, key: &dyn Reflect) -> Option<&dyn Reflect> {
        self.hash_table
            .find(Self::internal_hash(key), Self::internal_eq(key))
            .map(|(_, value)| &**value)
    }

    #[inline]
    fn get_mut(&mut self, key: &dyn Reflect) -> Option<&mut dyn Reflect> {
        self.hash_table
            .find_mut(Self::internal_hash(key), Self::internal_eq(key))
            .map(|(_, value)| &mut **value)
    }

    #[inline]
    fn len(&self) -> usize {
        self.hash_table.len()
    }

    #[inline]
    fn iter(&self) -> Box<dyn Iterator<Item = (&dyn Reflect, &dyn Reflect)> + '_> {
        let iter = self.hash_table.iter().map(|(k, v)| (&**k, &**v));
        Box::new(iter)
    }

    #[inline]
    fn drain(&mut self) -> Vec<(Box<dyn Reflect>, Box<dyn Reflect>)> {
        self.hash_table.drain().collect()
    }

    #[inline]
    fn retain(&mut self, f: &mut dyn FnMut(&dyn Reflect, &mut dyn Reflect) -> bool) {
        self.hash_table
            .retain(move |(key, value)| f(&**key, &mut **value));
    }

    fn insert_boxed(
        &mut self,
        key: Box<dyn Reflect>,
        value: Box<dyn Reflect>,
    ) -> Option<Box<dyn Reflect>> {
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

    fn remove(&mut self, key: &dyn Reflect) -> Option<Box<dyn Reflect>> {
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

    #[inline]
    fn reflect_map_info(&self) -> Option<&'static MapInfo> {
        None
    }

    #[inline]
    fn represented_map_info(&self) -> Option<&'static MapInfo> {
        self.map_info?.as_map().ok()
    }
}


/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn map_try_apply(x: &mut dyn Map, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_map()?;
    for (key, y_val) in y.iter() {
        if let Some(x_val) = x.get_mut(key) {
            x_val.try_apply(y_val)?;
        } else {
            x.insert_boxed(key.to_dynamic(), y_val.to_dynamic());
        }
    }
    x.retain(&mut |key, _| y.get(key).is_some());

    Ok(())
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn map_partial_eq(x: &dyn Map, y: &dyn Reflect) -> Option<bool> {
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

/// The default debug formatter for [`Map`] types.
/// 
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn map_debug(dyn_map: &dyn Map, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut debug = f.debug_map();
    for (key, value) in dyn_map.iter() {
        debug.entry(&key as &dyn fmt::Debug, &value as &dyn fmt::Debug);
    }
    debug.finish()
}
