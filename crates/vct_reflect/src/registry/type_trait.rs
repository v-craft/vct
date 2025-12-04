use alloc::boxed::Box;
use core::{
    any::{Any, TypeId},
    fmt,
    ops::{Deref, DerefMut},
};
use vct_utils::collections::TypeIdMap;

use crate::info::{TypeInfo, Typed};

pub trait TypeTrait: Any + Send + Sync {
    fn clone_type_trait(&self) -> Box<dyn TypeTrait>;
}

impl<T: Clone + Any + Send + Sync> TypeTrait for T {
    #[inline]
    fn clone_type_trait(&self) -> Box<dyn TypeTrait> {
        Box::new(self.clone())
    }
}

impl dyn TypeTrait {
    /// Returns `true` if the underlying value is of type `T`.
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        self.type_id() == TypeId::of::<T>()
    }

    /// Downcasts the value to type `T` by reference.
    #[inline]
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        <dyn Any>::downcast_ref(self)
    }

    /// Down
    #[inline]
    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        <dyn Any>::downcast_mut(self)
    }
}

pub struct TypeTraits {
    trait_map: TypeIdMap<Box<dyn TypeTrait>>,
    type_info: &'static TypeInfo,
}

impl TypeTraits {
    #[inline]
    pub fn of<T: Typed>() -> Self {
        Self {
            trait_map: TypeIdMap::new(),
            type_info: T::type_info(),
        }
    }

    #[inline]
    pub fn type_info(&self) -> &'static TypeInfo {
        self.type_info
    }

    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.type_info.type_id()
    }

    #[inline]
    pub fn insert<T: TypeTrait>(&mut self, data: T) {
        self.trait_map.insert(TypeId::of::<T>(), Box::new(data));
    }

    #[inline]
    pub fn remove<T: TypeTrait>(&mut self) -> Option<Box<T>> {
        // TODO: Use downcast_uncheck to reduce once type check
        // `Any::downcast_uncheck` is unstable now.
        self.trait_map
            .remove(&TypeId::of::<T>())
            .map(|val| <Box<dyn Any>>::downcast::<T>(val).unwrap())
    }

    #[inline]
    pub fn remove_by_id(&mut self, type_id: TypeId) -> Option<Box<dyn TypeTrait>> {
        self.trait_map.remove(&type_id)
    }

    #[inline]
    pub fn get<T: TypeTrait>(&self) -> Option<&T> {
        self.trait_map
            .get(&TypeId::of::<T>())
            .and_then(|val| val.downcast_ref::<T>())
    }

    #[inline]
    pub fn get_by_id(&self, type_id: TypeId) -> Option<&dyn TypeTrait> {
        self.trait_map.get(&type_id).map(Deref::deref)
    }

    #[inline]
    pub fn get_mut<T: TypeTrait>(&mut self) -> Option<&mut T> {
        self.trait_map
            .get_mut(&TypeId::of::<T>())
            .and_then(|val| val.downcast_mut())
    }

    #[inline]
    pub fn get_mut_by_id(&mut self, type_id: TypeId) -> Option<&mut dyn TypeTrait> {
        self.trait_map.get_mut(&type_id).map(DerefMut::deref_mut)
    }

    #[inline]
    pub fn contains<T: TypeTrait>(&self) -> bool {
        self.trait_map.contains_key(&TypeId::of::<T>())
    }

    #[inline]
    pub fn contains_by_id(&self, type_id: TypeId) -> bool {
        self.trait_map.contains_key(&type_id)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.trait_map.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.trait_map.is_empty()
    }

    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (TypeId, &dyn TypeTrait)> {
        self.trait_map.iter().map(|(key, val)| (*key, val.deref()))
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl ExactSizeIterator<Item = (TypeId, &mut dyn TypeTrait)> {
        self.trait_map
            .iter_mut()
            .map(|(key, val)| (*key, val.deref_mut()))
    }
}

impl Clone for TypeTraits {
    fn clone(&self) -> Self {
        let mut new_map = TypeIdMap::default();
        for (id, type_trait) in &self.trait_map {
            new_map.insert(*id, (*type_trait).clone_type_trait());
        }

        Self {
            trait_map: new_map,
            type_info: self.type_info,
        }
    }
}

impl fmt::Debug for TypeTraits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeTraits")
            .field("type_info", &self.type_info)
            .finish()
    }
}
