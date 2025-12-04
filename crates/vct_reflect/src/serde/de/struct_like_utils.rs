use alloc::{
    format,
    string::{String, ToString},
};
use core::{fmt, slice::Iter};
use serde::{
    Deserialize,
    de::{Error, MapAccess, SeqAccess, Visitor},
};

use crate::{
    info::{NamedField, StructInfo, StructVariantInfo},
    ops::DynamicStruct,
    registry::{TypeRegistry, TypeTraitDefault},
    serde::SkipSerde,
};

use super::{DeserializerProcessor, InternalDeserializer};

/// A helper trait for accessing type information from struct-like types.
pub(super) trait StructLikeInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E>;
    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E>;
    fn field_len(&self) -> usize;
    fn iter_fields(&self) -> Iter<'_, NamedField>;
}

impl StructLikeInfo for StructInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E> {
        Self::field(self, name).ok_or_else(|| {
            Error::custom(format!(
                "no field named `{}` on struct `{}`",
                name,
                self.type_path(),
            ))
        })
    }

    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on struct `{}`",
                index,
                self.type_path(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    #[inline]
    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

impl StructLikeInfo for StructVariantInfo {
    fn field<E: Error>(&self, name: &str) -> Result<&NamedField, E> {
        Self::field(self, name).ok_or_else(|| {
            Error::custom(format!(
                "no field named `{}` on variant `{}`",
                name,
                self.name(),
            ))
        })
    }

    fn field_at<E: Error>(&self, index: usize) -> Result<&NamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on variant `{}`",
                index,
                self.name(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }

    #[inline]
    fn iter_fields(&self) -> Iter<'_, NamedField> {
        self.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Ident(pub String);

impl<'de> Deserialize<'de> for Ident {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct IdentVisitor;

        impl<'de> Visitor<'de> for IdentVisitor {
            type Value = Ident;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("identifier")
            }

            #[inline]
            fn visit_str<E: Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(Ident(value.to_string()))
            }

            #[inline]
            fn visit_string<E: Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(Ident(value))
            }
        }

        deserializer.deserialize_identifier(IdentVisitor)
    }
}

/// Deserializes a [struct-like] type from a mapping of fields, returning a [`DynamicStruct`].
///
/// [struct-like]: StructLikeInfo
pub(super) fn visit_struct<'de, T, V, P>(
    map: &mut V,
    info: &'static T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: MapAccess<'de>,
    P: DeserializerProcessor,
{
    let mut dynamic_struct = DynamicStruct::new();

    while let Some(Ident(key)) = map.next_key::<Ident>()? {
        let field_ty = info.field::<V::Error>(&key)?.ty();
        let Some(type_traits) = registry.get(field_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{field_ty:?}`"
            )));
        };

        let value = map.next_value_seed(InternalDeserializer::new_internal(
            type_traits,
            registry,
            processor.as_deref_mut(),
        ))?;
        dynamic_struct.insert_boxed(key, value);
    }

    // skip serde fields
    for field in info.iter_fields() {
        if let Some(skip_serde) = field.get_attribute::<SkipSerde>() {
            if let Some(default_value) = &skip_serde.0 {
                if default_value.type_id() != field.type_id() {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not match the actual type `{}`.",
                        default_value.reflect_type_path(),
                        field.type_path(),
                    )));
                }
                if let Ok(val) = default_value.reflect_clone() {
                    dynamic_struct.insert_boxed(field.name(), val);
                } else {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not support `reflect_clone`.",
                        field.type_path(),
                    )));
                }
            } else {
                let field_ty = field.ty();
                let Some(type_traits) = registry.get(field_ty.id()) else {
                    return Err(Error::custom(format!(
                        "no type_traits found for type `{field_ty:?}`"
                    )));
                };
                if let Some(default_value) = type_traits.get::<TypeTraitDefault>() {
                    dynamic_struct.insert_boxed(field.name(), default_value.default());
                } else {
                    return Err(Error::custom(format!(
                        "Field `{}` skipped serde, but no default value and not support `TypeTraitDefault`.",
                        field.name(),
                    )));
                }
            }
        }
    }

    Ok(dynamic_struct)
}

/// Deserializes a [struct-like] type from a sequence of fields, returning a [`DynamicStruct`].
///
/// [struct-like]: StructLikeInfo
pub(super) fn visit_struct_seq<'de, T, V, P>(
    seq: &mut V,
    info: &T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicStruct, V::Error>
where
    T: StructLikeInfo,
    V: SeqAccess<'de>,
    P: DeserializerProcessor,
{
    let mut dynamic_struct = DynamicStruct::new();

    let len = info.field_len();

    for index in 0..len {
        let field_info = info.field_at::<V::Error>(index)?;
        let name = field_info.name();

        let Some(type_traits) = registry.get(field_info.type_id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{:?}`",
                field_info.ty()
            )));
        };

        // skip serde fields
        if let Some(skip_serde) = field_info.get_attribute::<SkipSerde>() {
            if let Some(default_value) = &skip_serde.0 {
                if default_value.type_id() != field_info.type_id() {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not match the actual type `{}`.",
                        default_value.reflect_type_path(),
                        field_info.type_path(),
                    )));
                }
                if let Ok(val) = default_value.reflect_clone() {
                    dynamic_struct.insert_boxed(field_info.name(), val);
                } else {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not support `reflect_clone`.",
                        field_info.type_path(),
                    )));
                }
            } else {
                if let Some(default_value) = type_traits.get::<TypeTraitDefault>() {
                    dynamic_struct.insert_boxed(field_info.name(), default_value.default());
                } else {
                    return Err(Error::custom(format!(
                        "Field `{}` skipped serde, but no default value and not support `TypeTraitDefault`.",
                        field_info.name(),
                    )));
                }
            }
            continue;
        }

        let value = seq
            .next_element_seed(InternalDeserializer::new_internal(
                type_traits,
                registry,
                processor.as_deref_mut(),
            ))?
            .ok_or_else(|| Error::invalid_length(index, &len.to_string().as_str()))?;
        dynamic_struct.insert_boxed(name, value);
    }

    Ok(dynamic_struct)
}
