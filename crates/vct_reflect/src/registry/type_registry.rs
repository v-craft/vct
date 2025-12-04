use core::{any::TypeId, fmt};

use crate::{
    info::{TypeInfo, Typed},
    registry::{FromType, GetTypeTraits, TypeTrait, TypeTraits},
};
use vct_os::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};
use vct_utils::collections::{HashMap, HashSet, TypeIdMap, hash_map};

/// A registry of reflected types.
pub struct TypeRegistry {
    traits_map: TypeIdMap<TypeTraits>,
    type_path_to_id: HashMap<&'static str, TypeId>,
    type_name_to_id: HashMap<&'static str, TypeId>,
    ambiguous_names: HashSet<&'static str>,
}

impl TypeRegistry {
    /// Create a empty [`TypeRegistry`].
    #[inline]
    pub const fn empty() -> Self {
        Self {
            traits_map: TypeIdMap::new(),
            type_path_to_id: HashMap::<_, _>::new(),
            type_name_to_id: HashMap::<_, _>::new(),
            ambiguous_names: HashSet::new(),
        }
    }

    // # Validity
    // The type must **not** already exist.
    fn add_new_type_indices(
        type_traits: &TypeTraits,
        type_path_to_id: &mut HashMap<&'static str, TypeId>,
        type_name_to_id: &mut HashMap<&'static str, TypeId>,
        ambiguous_names: &mut HashSet<&'static str>,
    ) {
        let type_name = type_traits.type_info().type_path_table().name();

        // Check for duplicate names.
        // The type should **not** already exist.
        if type_name_to_id.contains_key(type_name) || ambiguous_names.contains(type_name) {
            type_name_to_id.remove(type_name);
            ambiguous_names.insert(type_name);
        } else {
            type_name_to_id.insert(type_name, type_traits.type_id());
        }
        // For new type, assuming that the full path cannot be duplicated.
        type_path_to_id.insert(type_traits.type_info().type_path(), type_traits.type_id());
    }

    // If key [`TypeId`] has already exist, the function will do nothing and return `false`.
    // If the key [`TypeId`] does not exist, the function will insert value and return `true`.
    fn register_internal(
        &mut self,
        type_id: TypeId,
        get_type_traits: impl FnOnce() -> TypeTraits,
    ) -> bool {
        match self.traits_map.entry(type_id) {
            hash_map::Entry::Occupied(_) => false, // duplicated
            hash_map::Entry::Vacant(entry) => {
                let type_traits = get_type_traits();
                Self::add_new_type_indices(
                    &type_traits,
                    &mut self.type_path_to_id,
                    &mut self.type_name_to_id,
                    &mut self.ambiguous_names,
                );
                entry.insert(type_traits);
                true
            }
        }
    }

    /// Try add or do nothing.
    ///
    /// The function will will check if `TypeTraits.type_id()` exists.  
    /// - If key [`TypeId`] has already exist, the function will do nothing and return `false`.
    /// - If the key [`TypeId`] does not exist, the function will insert value and return `true`.
    pub fn try_add_type_traits(&mut self, type_traits: TypeTraits) -> bool {
        self.register_internal(type_traits.type_id(), || type_traits)
    }

    /// Insert or **Overwrite** inner TypeTraits.
    ///
    /// The function will will check if `TypeTraits.type_id()` exists.  
    /// - If key [`TypeId`] has already exist, the value will be overwritten.
    ///   But full_path and type_name table will not be modified.  
    /// - If the key [`TypeId`] does not exist, the value will be inserted.
    ///   And type path will be inserted to full_path and type_name table.
    pub fn insert_type_traits(&mut self, type_traits: TypeTraits) {
        match self.traits_map.entry(type_traits.type_id()) {
            hash_map::Entry::Occupied(mut entry) => {
                *entry.get_mut() = type_traits;
                // entry.insert(type_traits);
            }
            hash_map::Entry::Vacant(entry) => {
                Self::add_new_type_indices(
                    &type_traits,
                    &mut self.type_path_to_id,
                    &mut self.type_name_to_id,
                    &mut self.ambiguous_names,
                );
                entry.insert(type_traits);
            }
        }
    }

    /// Create a new [`TypeRegistry`].
    ///
    /// This function will register some types by default,
    /// such as `u8`-`u128`, `i8`-`i128`, `usize`, and `isize`.
    pub fn new() -> Self {
        let mut registry = Self::empty();

        registry.register::<u8>();
        registry.register::<i8>();
        registry.register::<u16>();
        registry.register::<i16>();
        registry.register::<u32>();
        registry.register::<i32>();
        registry.register::<u64>();
        registry.register::<i64>();
        registry.register::<u128>();
        registry.register::<i128>();
        registry.register::<usize>();
        registry.register::<isize>();

        // TODO: bool String

        registry
    }

    /// Attempts to register the type `T` if it has not yet been registered already.
    ///
    /// Register [`GetTypeTraits::get_type_traits`] for `T`.
    ///
    /// This will also recursively register any type dependencies as specified by [`GetTypeTraits::register_dependencies`].
    pub fn register<T: GetTypeTraits>(&mut self) {
        if self.register_internal(TypeId::of::<T>(), T::get_type_traits) {
            T::register_dependencies(self);
        }
    }

    /// Attempts to register the referenced type `T` if it has not yet been registered.
    #[inline]
    pub fn register_by_val<T: GetTypeTraits>(&mut self, _: &T) {
        self.register::<T>();
    }

    /// Registers the type type_trait `D` for type `T`.
    ///
    /// Type `T` must be registered in advance.
    ///
    /// # Panic
    ///
    /// - Type 'T' is not registered.
    pub fn register_type_trait<T: Typed, D: TypeTrait + FromType<T>>(&mut self) {
        match self.traits_map.get_mut(&TypeId::of::<T>()) {
            Some(type_traits) => type_traits.insert(D::from_type()),
            None => panic!(
                "Called `TypeRegistry::register_type_trait`, but the type `{}` of type_trait `{}` without registering",
                T::type_path(),
                core::any::type_name::<D>(),
            ),
        }
    }

    /// Whether the type with given [`TypeId`] has been registered in this registry.
    #[inline]
    pub fn contains(&self, type_id: TypeId) -> bool {
        self.traits_map.contains_key(&type_id)
    }
    /// Returns a reference to the [`TypeTraits`] of the type with the given [`TypeId`].
    #[inline]
    pub fn get(&self, type_id: TypeId) -> Option<&TypeTraits> {
        self.traits_map.get(&type_id)
    }

    /// Returns a mutable reference to the [`TypeTraits`] of the type with the given [`TypeId`].
    #[inline]
    pub fn get_mut(&mut self, type_id: TypeId) -> Option<&mut TypeTraits> {
        self.traits_map.get_mut(&type_id)
    }

    /// Returns a reference to the [`TypeTraits`] of the type with the given [type path].
    ///
    /// [type path]: TypePath::type_path
    pub fn get_with_type_path(&self, type_path: &str) -> Option<&TypeTraits> {
        // Manual inline
        match self.type_path_to_id.get(type_path) {
            Some(id) => self.get(*id),
            None => None,
        }
    }

    /// Returns a mutable reference to the [`TypeTraits`] of the type with the given [type path].
    ///
    /// [type path]: TypePath::type_path
    pub fn get_with_type_path_mut(&mut self, type_path: &str) -> Option<&mut TypeTraits> {
        // Manual inline
        match self.type_path_to_id.get(type_path) {
            Some(id) => self.get_mut(*id),
            None => None,
        }
    }

    /// Returns a reference to the [`TypeTraits`] of the type with the given [type name].
    ///
    /// [type name]: TypePath::type_name
    pub fn get_with_type_name(&self, type_name: &str) -> Option<&TypeTraits> {
        match self.type_name_to_id.get(type_name) {
            Some(id) => self.get(*id),
            None => None,
        }
    }

    /// Returns a mutable reference to the [`TypeTraits`] of the type with the given [type name].
    ///
    /// [type name]: TypePath::type_name
    pub fn get_with_type_name_mut(&mut self, type_name: &str) -> Option<&mut TypeTraits> {
        match self.type_name_to_id.get(type_name) {
            Some(id) => self.get_mut(*id),
            None => None,
        }
    }

    /// Returns `true` if the given [type name] is ambiguous, that is, it matches multiple registered types.
    ///
    /// [type name]: TypePath::type_name
    pub fn is_ambiguous(&self, type_name: &str) -> bool {
        self.ambiguous_names.contains(type_name)
    }

    /// Returns a reference to the [`TypeTrait`] of type `T` associated with the given [`TypeId`].
    pub fn get_type_trait<T: TypeTrait>(&self, type_id: TypeId) -> Option<&T> {
        // Manual inline
        match self.get(type_id) {
            Some(type_traits) => type_traits.get::<T>(),
            None => None,
        }
    }

    /// Returns a mutable reference to the [`TypeTrait`] of type `T` associated with the given [`TypeId`].
    pub fn get_type_trait_mut<T: TypeTrait>(&mut self, type_id: TypeId) -> Option<&mut T> {
        // Manual inline
        match self.get_mut(type_id) {
            Some(type_traits) => type_traits.get_mut::<T>(),
            None => None,
        }
    }

    /// Returns the [`TypeInfo`] associated with the given [`TypeId`].
    pub fn get_type_info(&self, type_id: TypeId) -> Option<&'static TypeInfo> {
        self.get(type_id).map(TypeTraits::type_info)
    }

    /// Returns an iterator over the [`TypeTraits`]s of the registered types.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &TypeTraits> {
        self.traits_map.values()
    }

    /// Returns a mutable iterator over the [`TypeTraits`]s of the registered types.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut TypeTraits> {
        self.traits_map.values_mut()
    }

    /// Checks to see if the [`TypeTrait`] of type `T` is associated with each registered type,
    /// returning a ([`TypeTraits`], [`TypeTrait`]) iterator for all entries where data of that type was found.
    pub fn iter_with_trait<T: TypeTrait>(&self) -> impl Iterator<Item = (&TypeTraits, &T)> {
        self.traits_map.values().filter_map(|item| {
            let type_trait = item.get::<T>();
            type_trait.map(|t| (item, t))
        })
    }
}

impl Default for TypeRegistry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
pub struct TypeRegistryArc {
    /// The wrapped [`TypeRegistry`].
    pub internal: Arc<RwLock<TypeRegistry>>,
}

impl TypeRegistryArc {
    /// Takes a read lock on the underlying [`TypeRegistry`].
    pub fn read(&self) -> RwLockReadGuard<'_, TypeRegistry> {
        self.internal.read().unwrap_or_else(PoisonError::into_inner)
    }

    /// Takes a write lock on the underlying [`TypeRegistry`].
    pub fn write(&self) -> RwLockWriteGuard<'_, TypeRegistry> {
        self.internal
            .write()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

impl fmt::Debug for TypeRegistryArc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.internal
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .type_path_to_id
            .keys()
            .fmt(f)
    }
}
