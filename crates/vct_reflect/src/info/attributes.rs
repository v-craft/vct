use alloc::boxed::Box;
use core::{any::TypeId, fmt};
use vct_utils::collections::TypeIdMap;

use crate::Reflect;

/// Single custom attribute
struct CustomAttribute {
    value: Box<dyn Reflect>,
}

impl CustomAttribute {
    /// create new container
    #[inline]
    pub fn new<T: Reflect>(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    /// Retrieve reference to internal value with type `T`
    #[inline]
    pub fn value<T: Reflect>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }

    /// Retrieve reference to internal value
    #[inline]
    pub fn reflect_value(&self) -> &dyn Reflect {
        &*self.value
    }
}

impl fmt::Debug for CustomAttribute {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.reflect_debug(f)
    }
}

/// Container for recording custom attributes
#[derive(Default)]
pub struct CustomAttributes {
    attributes: TypeIdMap<CustomAttribute>,
}

impl CustomAttributes {
    #[inline]
    pub const fn new() -> Self {
        Self { attributes: TypeIdMap::new() }
    }
    
    /// Add attributes
    #[inline]
    pub fn with_attribute<T: Reflect>(mut self, value: T) -> Self {
        self.attributes
            .insert(TypeId::of::<T>(), CustomAttribute::new(value));
        self
    }

    /// Get the iterator of internal data
    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&TypeId, &dyn Reflect)> {
        self.attributes
            .iter()
            .map(|(key, val)| (key, val.reflect_value()))
    }

    /// Check if it contains a certain attribute
    #[inline]
    pub fn contains<T: Reflect>(&self) -> bool {
        self.attributes.contains_key(&TypeId::of::<T>())
    }

    /// Check if it contains a certain attribute
    #[inline]
    pub fn contains_by_id(&self, id: TypeId) -> bool {
        self.attributes.contains_key(&id)
    }

    /// Get specified attributes
    #[inline]
    pub fn get<T: Reflect>(&self) -> Option<&T> {
        self.attributes.get(&TypeId::of::<T>())?.value::<T>()
    }

    /// Get specified attributes
    #[inline]
    pub fn get_by_id(&self, id: TypeId) -> Option<&dyn Reflect> {
        Some(self.attributes.get(&id)?.reflect_value())
    }

    /// Get the number of internal attributes
    #[inline]
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// return `true` if inner is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

impl fmt::Debug for CustomAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Not inline: Debug allows for performance loss
        f.debug_set().entries(self.attributes.values()).finish()
    }
}

/// impl `custom_attributes` `get_attribute` `get_attribute_by_id`
/// `has_attribute` `has_attribute_by_id`
macro_rules! impl_custom_attributes_fn {
    ($field:ident) => {
        $crate::info::attributes::impl_custom_attributes_fn!(self => &self.$field);
    };
    ($self:ident => $expr:expr) => {
        /// Return its own CustomAttributes
        #[inline]
        pub fn custom_attributes($self: &Self) -> Option<&$crate::info::CustomAttributes> {
            match $expr {
                Some(arc) => Some(&**arc),
                None => todo!(),
            }
        }

        /// Get specified attributes
        pub fn get_attribute<T: $crate::Reflect>($self: &Self) -> Option<&T> {
            // Not inline: Avoid excessive inline (recursion)
            $self.custom_attributes()?.get::<T>()
        }

        /// Get specified attributes
        pub fn get_attribute_by_id($self: &Self, __id: ::core::any::TypeId) -> Option<&dyn $crate::Reflect> {
            // Not inline: Avoid excessive inline (recursion)
            $self.custom_attributes()?.get_by_id(__id)
        }

        /// Check if it contains a certain attribute
        pub fn has_attribute<T: $crate::Reflect>($self: &Self) -> bool {
            // Not inline: Avoid excessive inline (recursion)
            match $self.custom_attributes() {
                Some(attrs) => attrs.contains::<T>(),
                None => false,
            }
        }

        /// Check if it contains a certain attribute
        pub fn has_attribute_by_id($self: &Self, __id: ::core::any::TypeId) -> bool {
            // Not inline: Avoid excessive inline (recursion)
            match $self.custom_attributes() {
                Some(attrs) => attrs.contains_by_id(__id),
                None => false,
            }
        }
    };
}

macro_rules! impl_with_custom_attributes {
    ($field:ident) => {
        /// Modify attributes (overwrite, not add)
        pub fn with_custom_attributes(self, attributes: CustomAttributes) -> Self {
            if attributes.is_empty() {
                Self {
                    $field: None,
                    ..self
                }
            } else {
                Self {
                    $field: Some(Arc::new(attributes)),
                    ..self
                }
            }
        }
    };
}

pub(crate) use impl_custom_attributes_fn;
pub(crate) use impl_with_custom_attributes;
