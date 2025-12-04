use alloc::format;
use serde::{Serialize, Serializer, ser::SerializeMap};

use crate::{
    Reflect,
    ops::ReflectRef,
    registry::{TypeRegistry, TypeTraitSerialize},
};

use super::{
    SerializerProcessor, array_serializer::ArraySerializer, enum_serializer::EnumSerializer,
    list_serializer::ListSerializer, map_serializer::MapSerializer, set_serializer::SetSerializer,
    struct_serializer::StructSerializer, tuple_serializer::TupleSerializer,
    tuple_struct_serializer::TupleStructSerializer,
};

/// A serializer without type path attached
pub struct InternalSerializer<'a, P: SerializerProcessor = ()> {
    value: &'a dyn Reflect,
    registry: &'a TypeRegistry,
    processor: Option<&'a P>,
}

impl<'a> InternalSerializer<'a, ()> {
    /// Creates a serializer with no processor.
    ///
    /// If you want to add custom logic for serializing certain values, use
    /// [`with_processor`](Self::with_processor).
    #[inline]
    pub fn new(value: &'a dyn Reflect, registry: &'a TypeRegistry) -> Self {
        Self {
            value,
            registry,
            processor: None,
        }
    }
}

impl<'a, P: SerializerProcessor> InternalSerializer<'a, P> {
    /// Creates a serializer with a processor.
    #[inline]
    pub fn with_processor(
        value: &'a dyn Reflect,
        registry: &'a TypeRegistry,
        processor: &'a P,
    ) -> Self {
        Self {
            value,
            registry,
            processor: Some(processor),
        }
    }

    #[inline]
    pub(super) fn new_internal(
        value: &'a dyn Reflect,
        registry: &'a TypeRegistry,
        processor: Option<&'a P>,
    ) -> Self {
        Self {
            value,
            registry,
            processor,
        }
    }
}

impl<'a, P: SerializerProcessor> Serialize for InternalSerializer<'a, P> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let serializer = if let Some(processor) = self.processor {
            match processor.try_serialize(self.value, self.registry, serializer) {
                Ok(result) => return result,
                Err(serializer) => serializer, // Not support serialize, it's not a error.
            }
        } else {
            serializer
        };

        // Try to get the Serializ impl of the type itself
        if let Some(type_traits) = self.registry.get(self.value.type_id())
            && let Some(processor) = type_traits.get::<TypeTraitSerialize>()
        {
            return processor.serialize(self.value, serializer);
        }

        match self.value.reflect_ref() {
            ReflectRef::Struct(struct_value) => StructSerializer {
                struct_value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::TupleStruct(tuple_struct) => TupleStructSerializer {
                tuple_struct,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Tuple(tuple) => TupleSerializer {
                tuple,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::List(list) => ListSerializer {
                list,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Array(array) => ArraySerializer {
                array,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Map(map) => MapSerializer {
                map,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Set(set) => SetSerializer {
                set,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Enum(enum_value) => EnumSerializer {
                enum_value,
                registry: self.registry,
                processor: self.processor,
            }
            .serialize(serializer),
            ReflectRef::Opaque(_) => Err(serde::ser::Error::custom(
                "No serialization method available for this Opauqe was found.",
            )),
        }
    }
}

/// A serializer with type path attached
pub struct ReflectSerializer<'a, P: SerializerProcessor = ()> {
    value: &'a dyn Reflect,
    registry: &'a TypeRegistry,
    processor: Option<&'a P>,
}

impl<'a> ReflectSerializer<'a, ()> {
    /// Creates a serializer with no processor.
    ///
    /// If you want to add custom logic for serializing certain values, use
    /// [`with_processor`](Self::with_processor).
    #[inline]
    pub fn new(value: &'a dyn Reflect, registry: &'a TypeRegistry) -> Self {
        Self {
            value,
            registry,
            processor: None,
        }
    }
}

impl<'a, P: SerializerProcessor> ReflectSerializer<'a, P> {
    /// Creates a serializer with a processor.
    ///
    /// If you do not need any custom logic for handling certain values, use
    /// [`new`](Self::new).
    #[inline]
    pub fn with_processor(
        value: &'a dyn Reflect,
        registry: &'a TypeRegistry,
        processor: &'a P,
    ) -> Self {
        Self {
            value,
            registry,
            processor: Some(processor),
        }
    }
}

impl<P: SerializerProcessor> Serialize for ReflectSerializer<'_, P> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_map(Some(1))?;
        state.serialize_entry(
            self.value
                .represented_type_info()
                .ok_or_else(|| {
                    if self.value.is_dynamic() {
                        serde::ser::Error::custom(format!(
                            "cannot get represented type from dynamic type: `{}`.",
                            self.value.reflect_type_path(),
                        ))
                    } else {
                        serde::ser::Error::custom(format!(
                            "cannot get type info for `{}`.",
                            self.value.reflect_type_path(),
                        ))
                    }
                })?
                .type_path(),
            &InternalSerializer::new_internal(self.value, self.registry, self.processor),
        )?;
        state.end()
    }
}
