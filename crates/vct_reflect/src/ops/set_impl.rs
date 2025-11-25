use crate::{
    PartialReflect, Reflect,
    info::{MaybeTyped, ReflectKind, SetInfo, TypeInfo, TypePath},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
};
use alloc::{boxed::Box, format, vec::Vec};
use core::fmt;
use vct_utils::collections::{HashTable, hash_table};

/// An unordered set of reflected values.
#[derive(Default)]
pub struct DynamicSet {
    target_type: Option<&'static TypeInfo>,
    hash_table: HashTable<Box<dyn PartialReflect>>,
}

impl TypePath for DynamicSet {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicSet"
    }

    #[inline]
    fn short_type_path() -> &'static str {
        "DynamicSet"
    }

    #[inline]
    fn type_ident() -> Option<&'static str> {
        Some("DynamicSet")
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

impl DynamicSet {
    /// Sets the [`TypeInfo`] to be represented by this `DynamicSet`.
    ///
    /// # Panics
    ///
    /// Panics if the given [`TypeInfo`] is not a [`TypeInfo::Set`].
    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::Set(_)),
                "expected TypeInfo::Set but received: {target_type:?}"
            );
        }

        self.target_type = target_type;
    }

    /// Inserts a typed value into the set.
    #[inline]
    pub fn insert<V: Reflect>(&mut self, value: V) {
        self.insert_boxed(Box::new(value));
    }

    fn internal_hash(value: &dyn PartialReflect) -> u64 {
        value.reflect_hash().expect(&{
            let type_path = (value).reflect_type_path();
            if !value.is_dynamic() {
                format!(
                    "the given value of type `{}` does not support hashing",
                    type_path
                )
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
        value: &dyn PartialReflect,
    ) -> impl FnMut(&Box<dyn PartialReflect>) -> bool + '_ {
        |other| {
            value
                .reflect_partial_eq(&**other)
                .expect("Underlying type does not reflect `PartialEq` and hence doesn't support equality checks")
        }
    }
}

impl PartialReflect for DynamicSet {
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
        let other = value.reflect_ref().as_set()?;

        for other_val in other.iter() {
            if !self.contains(other_val) {
                self.insert_boxed(other_val.to_dynamic());
            }
        }
        self.retain(&mut |val| other.contains(val));
        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Set
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Set(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Set(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Set(self)
    }

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        // Not Inline: `set_partial_eq()` is inline always
        set_partial_eq(self, other)
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicSet(")?;
        set_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicSet {}

impl fmt::Debug for DynamicSet {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl FromIterator<Box<dyn PartialReflect>> for DynamicSet {
    fn from_iter<I: IntoIterator<Item = Box<dyn PartialReflect>>>(values: I) -> Self {
        // for compile-time runing
        let mut this = Self {
            target_type: None,
            hash_table: HashTable::new(),
        };

        for value in values {
            this.insert_boxed(value);
        }

        this
    }
}

impl<T: Reflect> FromIterator<T> for DynamicSet {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        // for compile-time runing
        let mut this = Self {
            target_type: None,
            hash_table: HashTable::new(),
        };

        for value in values {
            this.insert(value);
        }

        this
    }
}

impl IntoIterator for DynamicSet {
    type Item = Box<dyn PartialReflect>;
    type IntoIter = hash_table::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicSet {
    type Item = &'a dyn PartialReflect;
    type IntoIter = core::iter::Map<
        hash_table::Iter<'a, Box<dyn PartialReflect>>,
        fn(&'a Box<dyn PartialReflect>) -> Self::Item,
    >;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table.iter().map(|v| v.as_ref())
    }
}

/// A trait used to power [set-like] operations via [reflection].
///
/// Sets contain zero or more entries of a fixed type, and correspond to types
/// like `HashSet` and `BTreeSet`.
/// The order of these entries is not guaranteed by this trait.
pub trait Set: PartialReflect {
    /// Returns a reference to the value.
    ///
    /// If no value is contained, returns `None`.
    fn get(&self, value: &dyn PartialReflect) -> Option<&dyn PartialReflect>;

    /// Returns the number of elements in the set.
    fn len(&self) -> usize;

    /// Returns `true` if the list contains no elements.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the values of the set.
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn PartialReflect> + '_>;

    /// Drain the values of this set to get a vector of owned values.
    ///
    /// After calling this function, `self` will be empty.
    fn drain(&mut self) -> Vec<Box<dyn PartialReflect>>;

    /// Retain only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    fn retain(&mut self, f: &mut dyn FnMut(&dyn PartialReflect) -> bool);

    /// Creates a new [`DynamicSet`] from this set.
    fn to_dynamic_set(&self) -> DynamicSet {
        let mut set = DynamicSet::default();
        set.set_target_type_info(self.get_target_type_info());
        for value in self.iter() {
            set.insert_boxed(value.to_dynamic());
        }
        set
    }

    /// Inserts a value into the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    /// If the set did have this value present, `false` is returned.
    fn insert_boxed(&mut self, value: Box<dyn PartialReflect>) -> bool;

    /// Removes a value from the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    /// If the set did have this value present, `false` is returned.
    fn remove(&mut self, value: &dyn PartialReflect) -> bool;

    /// Checks if the given value is contained in the set
    fn contains(&self, value: &dyn PartialReflect) -> bool;

    /// Will return `None` if [`TypeInfo`] is not available.
    #[inline]
    fn get_target_set_info(&self) -> Option<&'static SetInfo> {
        self.get_target_type_info()?.as_set().ok()
    }
}

impl Set for DynamicSet {
    #[inline]
    fn get(&self, value: &dyn PartialReflect) -> Option<&dyn PartialReflect> {
        self.hash_table
            .find(Self::internal_hash(value), Self::internal_eq(value))
            .map(|value| &**value)
    }

    #[inline]
    fn len(&self) -> usize {
        self.hash_table.len()
    }

    #[inline]
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn PartialReflect> + '_> {
        Box::new(self.hash_table.iter().map(|v| &**v))
    }

    #[inline]
    fn drain(&mut self) -> Vec<Box<dyn PartialReflect>> {
        self.hash_table.drain().collect::<Vec<_>>()
    }

    #[inline]
    fn retain(&mut self, f: &mut dyn FnMut(&dyn PartialReflect) -> bool) {
        self.hash_table.retain(move |value| f(&**value));
    }

    fn insert_boxed(&mut self, value: Box<dyn PartialReflect>) -> bool {
        assert_eq!(
            value.reflect_partial_eq(&*value),
            Some(true),
            "Values inserted in `Set` like types are expected to reflect `PartialEq`"
        );
        match self
            .hash_table
            .find_mut(Self::internal_hash(&*value), Self::internal_eq(&*value))
        {
            Some(old) => {
                *old = value;
                false
            }
            None => {
                self.hash_table.insert_unique(
                    Self::internal_hash(value.as_ref()),
                    value,
                    |boxed| Self::internal_hash(boxed.as_ref()),
                );
                true
            }
        }
    }

    #[inline]
    fn remove(&mut self, value: &dyn PartialReflect) -> bool {
        self.hash_table
            .find_entry(Self::internal_hash(value), Self::internal_eq(value))
            .map(hash_table::OccupiedEntry::remove)
            .is_ok()
    }

    #[inline]
    fn contains(&self, value: &dyn PartialReflect) -> bool {
        self.hash_table
            .find(Self::internal_hash(value), Self::internal_eq(value))
            .is_some()
    }
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// It's `inline(always)`, Usually recommended only for impl `reflect_partial_eq`.
#[inline(always)]
pub fn set_partial_eq<M: Set>(x: &M, y: &dyn PartialReflect) -> Option<bool> {
    // Inline: this function **should only** be used to impl `PartialReflect::reflect_partial_eq`
    // Compilation times is related to the quantity of type A.
    // Therefore, inline has no negative effects.
    let ReflectRef::Set(y) = y.reflect_ref() else {
        return Some(false);
    };
    if x.len() != y.len() {
        return Some(false);
    }

    for val in x.iter() {
        if let Some(y_val) = y.get(val) {
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

/// The default debug formatter for [`Set`] types.
pub fn set_debug(dyn_set: &dyn Set, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `PartialReflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_set();
    for value in dyn_set.iter() {
        debug.entry(&value as &dyn fmt::Debug);
    }
    debug.finish()
}
