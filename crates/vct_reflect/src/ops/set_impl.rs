use crate::{
    Reflect,
    cell::NonGenericTypeInfoCell,
    info::{ReflectKind, SetInfo, TypeInfo, TypePath, Typed, OpaqueInfo},
    ops::{ApplyError, ReflectMut, ReflectOwned, ReflectRef},
    reflect::impl_cast_reflect_fn,
};
use alloc::{boxed::Box, format, vec::Vec};
use core::fmt;
use vct_utils::collections::{HashTable, hash_table};

/// Representing [`Set`]`, used to dynamically modify the type of data and information.
/// 
/// Dynamic types are special in that their TypeInfo is [`OpaqueInfo`], 
/// but other APIs are consistent with the type they represent, such as [`reflect_kind`], [`reflect_ref`]
/// 
/// [`reflect_kind`]: crate::Reflect::reflect_kind
/// [`reflect_ref`]: crate::Reflect::reflect_ref
#[derive(Default)]
pub struct DynamicSet {
    set_info: Option<&'static TypeInfo>,
    hash_table: HashTable<Box<dyn Reflect>>,
}

impl TypePath for DynamicSet {
    #[inline]
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicSet"
    }

    #[inline]
    fn type_name() -> &'static str {
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

impl Typed for DynamicSet {
    fn type_info() -> &'static TypeInfo {
        static CELL: NonGenericTypeInfoCell = NonGenericTypeInfoCell::new();
        CELL.get_or_init(|| TypeInfo::Opaque(OpaqueInfo::new::<Self>()))
    }
}

impl DynamicSet {
    #[inline]
    pub const fn new() -> Self {
        Self { set_info: None, hash_table: HashTable::new() }
    }


    /// Sets the [`TypeInfo`] to be represented by this `DynamicSet`.
    /// 
    /// # Panic
    /// 
    /// If the input is not list info or None.
    #[inline]
    pub fn set_type_info(&mut self, set_info: Option<&'static TypeInfo>) {
        match set_info {
            Some(TypeInfo::Set(_)) | None => {},
            _ => { panic!("Call `DynamicSet::set_type_info`, but the input is not set information or None.") },
        }

        self.set_info = set_info;
    }

    /// Inserts a typed value into the set.
    #[inline]
    pub fn insert<V: Reflect>(&mut self, value: V) {
        self.insert_boxed(Box::new(value));
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
        value: &dyn Reflect,
    ) -> impl FnMut(&Box<dyn Reflect>) -> bool + '_ {
        |other| {
            value
                .reflect_partial_eq(&**other)
                .expect("Underlying type does not reflect `PartialEq` and hence doesn't support equality checks")
        }
    }
}

impl Reflect for DynamicSet {
    impl_cast_reflect_fn!();

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }

    #[inline]
    fn represented_type_info(&self) -> Option<&'static TypeInfo> {
        self.set_info
    }

    #[inline]
    fn try_apply(&mut self, value: &dyn Reflect) -> Result<(), ApplyError> {
        set_try_apply(self, value)
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

    #[inline]
    fn reflect_partial_eq(&self, other: &dyn Reflect) -> Option<bool> {
        set_partial_eq(self, other)
    }

    #[inline]
    fn reflect_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicSet(")?;
        set_debug(self, f)?;
        write!(f, ")")
    }

}

impl fmt::Debug for DynamicSet {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.reflect_debug(f)
    }
}

impl FromIterator<Box<dyn Reflect>> for DynamicSet {
    fn from_iter<I: IntoIterator<Item = Box<dyn Reflect>>>(values: I) -> Self {
        let mut this = DynamicSet::new();

        for value in values {
            this.insert_boxed(value);
        }

        this
    }
}

impl<T: Reflect> FromIterator<T> for DynamicSet {
    fn from_iter<I: IntoIterator<Item = T>>(values: I) -> Self {
        let mut this = DynamicSet::new();

        for value in values {
            this.insert(value);
        }

        this
    }
}

impl IntoIterator for DynamicSet {
    type Item = Box<dyn Reflect>;
    type IntoIter = hash_table::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.hash_table.into_iter()
    }
}

impl<'a> IntoIterator for &'a DynamicSet {
    type Item = &'a dyn Reflect;
    type IntoIter = core::iter::Map<
        hash_table::Iter<'a, Box<dyn Reflect>>,
        fn(&'a Box<dyn Reflect>) -> Self::Item,
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
pub trait Set: Reflect {
    /// Returns a reference to the value.
    ///
    /// If no value is contained, returns `None`.
    fn get(&self, value: &dyn Reflect) -> Option<&dyn Reflect>;

    /// Returns the number of elements in the set.
    fn len(&self) -> usize;

    /// Returns `true` if the list contains no elements.
    #[inline]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the values of the set.
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Reflect> + '_>;

    /// Drain the values of this set to get a vector of owned values.
    ///
    /// After calling this function, `self` will be empty.
    fn drain(&mut self) -> Vec<Box<dyn Reflect>>;

    /// Retain only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    fn retain(&mut self, f: &mut dyn FnMut(&dyn Reflect) -> bool);

    /// Creates a new [`DynamicSet`] from this set.
    fn to_dynamic_set(&self) -> DynamicSet {
        let mut set = DynamicSet::default();
        set.set_type_info(self.represented_type_info());
        for value in self.iter() {
            set.insert_boxed(value.to_dynamic());
        }
        set
    }

    /// Inserts a value into the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    /// If the set did have this value present, `false` is returned.
    fn insert_boxed(&mut self, value: Box<dyn Reflect>) -> bool;

    /// Removes a value from the set.
    ///
    /// If the set did not have this value present, `true` is returned.
    /// If the set did have this value present, `false` is returned.
    fn remove(&mut self, value: &dyn Reflect) -> bool;

    /// Checks if the given value is contained in the set
    fn contains(&self, value: &dyn Reflect) -> bool;

    /// Get actual [`SetInfo`] of underlying types.
    /// 
    /// If it is a dynamic type, it will return `None`.
    /// 
    /// If it is not a dynamic type and the returned value is not `None` or `SetInfo`, it will panic.
    /// (If you want to implement dynamic types yourself, please return None.)
    #[inline]
    fn reflect_set_info(&self) -> Option<&'static SetInfo> {
        self.reflect_type_info().as_set().ok()
    }

    /// Get the [`SetInfo`] of representation.
    /// 
    /// Normal types return their own information,
    /// while dynamic types return `None`` if they do not represent an object
    #[inline]
    fn represented_set_info(&self) -> Option<&'static SetInfo> {
        self.represented_type_info()?.as_set().ok()
    }
}

impl Set for DynamicSet {
    #[inline]
    fn get(&self, value: &dyn Reflect) -> Option<&dyn Reflect> {
        self.hash_table
            .find(Self::internal_hash(value), Self::internal_eq(value))
            .map(|value| &**value)
    }

    #[inline]
    fn len(&self) -> usize {
        self.hash_table.len()
    }

    #[inline]
    fn iter(&self) -> Box<dyn Iterator<Item = &dyn Reflect> + '_> {
        Box::new(self.hash_table.iter().map(|v| &**v))
    }

    #[inline]
    fn drain(&mut self) -> Vec<Box<dyn Reflect>> {
        self.hash_table.drain().collect::<Vec<_>>()
    }

    #[inline]
    fn retain(&mut self, f: &mut dyn FnMut(&dyn Reflect) -> bool) {
        self.hash_table.retain(move |value| f(&**value));
    }

    fn insert_boxed(&mut self, value: Box<dyn Reflect>) -> bool {
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
    fn remove(&mut self, value: &dyn Reflect) -> bool {
        self.hash_table
            .find_entry(Self::internal_hash(value), Self::internal_eq(value))
            .map(hash_table::OccupiedEntry::remove)
            .is_ok()
    }

    #[inline]
    fn contains(&self, value: &dyn Reflect) -> bool {
        self.hash_table
            .find(Self::internal_hash(value), Self::internal_eq(value))
            .is_some()
    }
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn set_try_apply(x: &mut dyn Set, y: &dyn Reflect) -> Result<(), ApplyError> {
    let y = y.reflect_ref().as_set()?;

    for y_val in y.iter() {
        if !x.contains(y_val) {
            x.insert_boxed(y_val.to_dynamic());
        }
    }
    x.retain(&mut |val| y.contains(val));
    Ok(())
}

/// A function used to assist in the implementation of `reflect_partial_eq`
///
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub fn set_partial_eq(x: &dyn Set, y: &dyn Reflect) -> Option<bool> {
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
/// 
/// Avoid compilation overhead when implementing multiple types.
#[inline(never)]
pub(crate) fn set_debug(dyn_set: &dyn Set, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // This function should only be used to impl `PartialReflect::debug`
    // Non Inline: only be compiled once -> reduce compilation times
    let mut debug = f.debug_set();
    for value in dyn_set.iter() {
        debug.entry(&value as &dyn fmt::Debug);
    }
    debug.finish()
}
