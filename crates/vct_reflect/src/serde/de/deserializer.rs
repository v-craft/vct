use alloc::{boxed::Box, format};
use core::fmt;
use serde::{
    Deserializer,
    de::{DeserializeSeed, Error, IgnoredAny, MapAccess, Visitor},
};

use crate::{
    Reflect,
    info::{TypeInfo, Typed},
    registry::{
        GetTypeTraits, TypeRegistry, TypeTraitDeserialize, TypeTraitFromReflect, TypeTraits,
    },
};

use super::{
    DeserializerProcessor, array_visitor::ArrayVisitor, enum_visitor::EnumVisitor,
    list_visitor::ListVisitor, map_visitor::MapVisitor, option_visitor::OptionVisitor,
    set_visitor::SetVisitor, struct_visitor::StructVisitor,
    tuple_struct_visitor::TupleStructVisitor, tuple_visitor::TupleVisitor,
};

pub struct InternalDeserializer<'a, P: DeserializerProcessor = ()> {
    type_traits: &'a TypeTraits,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut P>,
}

impl<'a> InternalDeserializer<'a, ()> {
    #[inline]
    pub fn new(type_traits: &'a TypeTraits, registry: &'a TypeRegistry) -> Self {
        Self {
            type_traits,
            registry,
            processor: None,
        }
    }

    #[inline]
    pub fn of<T: Typed + GetTypeTraits>(registry: &'a TypeRegistry) -> Self {
        let type_traits = registry
            .get(core::any::TypeId::of::<T>())
            .unwrap_or_else(|| panic!("no type_traits found for type `{}`", T::type_path()));

        Self {
            type_traits,
            registry,
            processor: None,
        }
    }
}

impl<'a, P: DeserializerProcessor> InternalDeserializer<'a, P> {
    #[inline]
    pub fn with_processor(
        type_traits: &'a TypeTraits,
        registry: &'a TypeRegistry,
        processor: &'a mut P,
    ) -> Self {
        Self {
            type_traits,
            registry,
            processor: Some(processor),
        }
    }

    /// An internal constructor for creating a deserializer without resetting the type info stack.
    #[inline]
    pub(super) fn new_internal(
        type_traits: &'a TypeTraits,
        registry: &'a TypeRegistry,
        processor: Option<&'a mut P>,
    ) -> Self {
        Self {
            type_traits,
            registry,
            processor,
        }
    }
}

impl<'de, P: DeserializerProcessor> DeserializeSeed<'de> for InternalDeserializer<'_, P> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D: Deserializer<'de>>(
        mut self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        let deserializer = if let Some(processor) = self.processor.as_deref_mut() {
            match processor.try_deserialize(self.type_traits, self.registry, deserializer) {
                Ok(Ok(value)) => return Ok(value),
                Ok(Err(err)) => return Err(err),
                Err(deserializer) => deserializer,
            }
        } else {
            deserializer
        };

        if let Some(deserialize_reflect) = self.type_traits.get::<TypeTraitDeserialize>() {
            return deserialize_reflect.deserialize(deserializer);
        }

        let dynamic_value: Box<dyn Reflect> = match self.type_traits.type_info() {
            TypeInfo::Struct(struct_info) => {
                let mut dynamic_struct = deserializer.deserialize_struct(
                    struct_info
                        .type_path_table()
                        .ident()
                        .unwrap_or(crate::serde::NO_IDENT),
                    struct_info.field_names(),
                    StructVisitor {
                        struct_info,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?;
                dynamic_struct.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_struct)
            }
            TypeInfo::TupleStruct(tuple_struct_info) => {
                let mut dynamic_tuple_struct = if tuple_struct_info.field_len() == 1 {
                    deserializer.deserialize_newtype_struct(
                        tuple_struct_info
                            .type_path_table()
                            .ident()
                            .unwrap_or(crate::serde::NO_IDENT),
                        TupleStructVisitor {
                            tuple_struct_info,
                            registry: self.registry,
                            processor: self.processor,
                        },
                    )?
                } else {
                    deserializer.deserialize_tuple_struct(
                        tuple_struct_info
                            .type_path_table()
                            .ident()
                            .unwrap_or(crate::serde::NO_IDENT),
                        tuple_struct_info.field_len(),
                        TupleStructVisitor {
                            tuple_struct_info,
                            registry: self.registry,
                            processor: self.processor,
                        },
                    )?
                };
                dynamic_tuple_struct.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_tuple_struct)
            }
            TypeInfo::Tuple(tuple_info) => {
                let mut dynamic_tuple = deserializer.deserialize_tuple(
                    tuple_info.field_len(),
                    TupleVisitor {
                        tuple_info,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?;
                dynamic_tuple.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_tuple)
            }
            TypeInfo::List(list_info) => {
                let mut dynamic_list = deserializer.deserialize_seq(ListVisitor {
                    list_info,
                    registry: self.registry,
                    processor: self.processor,
                })?;
                dynamic_list.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_list)
            }
            TypeInfo::Array(array_info) => {
                let mut dynamic_array = deserializer.deserialize_tuple(
                    array_info.capacity(),
                    ArrayVisitor {
                        array_info,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?;
                dynamic_array.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_array)
            }
            TypeInfo::Map(map_info) => {
                let mut dynamic_map = deserializer.deserialize_map(MapVisitor {
                    map_info,
                    registry: self.registry,
                    processor: self.processor,
                })?;
                dynamic_map.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_map)
            }
            TypeInfo::Set(set_info) => {
                let mut dynamic_set = deserializer.deserialize_seq(SetVisitor {
                    set_info,
                    registry: self.registry,
                    processor: self.processor,
                })?;
                dynamic_set.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_set)
            }
            TypeInfo::Enum(enum_info) => {
                let mut dynamic_enum = if enum_info.type_path_table().module_path()
                    == Some("core::option")
                    && enum_info.type_path_table().ident() == Some("Option")
                {
                    deserializer.deserialize_option(OptionVisitor {
                        enum_info,
                        registry: self.registry,
                        processor: self.processor,
                    })?
                } else {
                    deserializer.deserialize_enum(
                        enum_info.type_path_table().ident().unwrap(),
                        enum_info.variant_names(),
                        EnumVisitor {
                            enum_info,
                            registry: self.registry,
                            processor: self.processor,
                        },
                    )?
                };
                dynamic_enum.set_type_info(Some(self.type_traits.type_info()));
                Box::new(dynamic_enum)
            }
            TypeInfo::Opaque(_) => {
                return Err(Error::custom(
                    "No deserialization method available for this Opauqe was found.",
                ));
            }
        };

        if let Some(from_reflect) = self.type_traits.get::<TypeTraitFromReflect>()
            && let Some(value) = from_reflect.from_reflect(&*dynamic_value)
        {
            return Ok(value);
        }

        Ok(dynamic_value)
    }
}

pub struct ReflectDeserializer<'a, P: DeserializerProcessor = ()> {
    registry: &'a TypeRegistry,
    processor: Option<&'a mut P>,
}

impl<'a> ReflectDeserializer<'a, ()> {
    #[inline]
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self {
            registry,
            processor: None,
        }
    }
}

impl<'a, P: DeserializerProcessor> ReflectDeserializer<'a, P> {
    #[inline]
    pub fn with_processor(registry: &'a TypeRegistry, processor: &'a mut P) -> Self {
        Self {
            registry,
            processor: Some(processor),
        }
    }
}

impl<'de, P: DeserializerProcessor> DeserializeSeed<'de> for ReflectDeserializer<'_, P> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct ReflectDeserializerVisitor<'a, P> {
            registry: &'a TypeRegistry,
            processor: Option<&'a mut P>,
        }

        impl<'de, P: DeserializerProcessor> Visitor<'de> for ReflectDeserializerVisitor<'_, P> {
            type Value = Box<dyn Reflect>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter
                    .write_str("map containing `type` and `value` entries for the reflected value")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                // Get `TypeTraits` from type_path
                let registration = map
                    .next_key_seed(TypePathDeserializer::new(self.registry))?
                    .ok_or_else(|| Error::invalid_length(0, &"a single entry"))?;

                let value = map.next_value_seed(InternalDeserializer::new_internal(
                    registration,
                    self.registry,
                    self.processor,
                ))?;

                if map.next_key::<IgnoredAny>()?.is_some() {
                    return Err(Error::invalid_length(2, &"a single entry"));
                }

                Ok(value)
            }
        }

        deserializer.deserialize_map(ReflectDeserializerVisitor {
            registry: self.registry,
            processor: self.processor,
        })
    }
}

pub struct TypePathDeserializer<'a> {
    registry: &'a TypeRegistry,
}

impl<'a> TypePathDeserializer<'a> {
    /// Creates a new [`TypePathDeserializer`].
    #[inline]
    pub fn new(registry: &'a TypeRegistry) -> Self {
        Self { registry }
    }
}

impl<'a, 'de> DeserializeSeed<'de> for TypePathDeserializer<'a> {
    type Value = &'a TypeTraits;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        struct TypePathVisitor<'a>(&'a TypeRegistry);

        impl<'de, 'a> Visitor<'de> for TypePathVisitor<'a> {
            type Value = &'a TypeTraits;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("string containing `type` entry for the reflected value")
            }

            fn visit_str<E: Error>(self, type_path: &str) -> Result<Self::Value, E> {
                self.0.get_with_type_path(type_path).ok_or_else(|| {
                    Error::custom(format!("no registration found for `{type_path}`"))
                })
            }
        }

        deserializer.deserialize_str(TypePathVisitor(self.registry))
    }
}
