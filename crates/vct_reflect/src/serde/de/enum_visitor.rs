use alloc::format;
use core::{fmt, fmt::Formatter};
use serde::de::{DeserializeSeed, EnumAccess, Error, MapAccess, SeqAccess, VariantAccess, Visitor};

use crate::{
    info::{EnumInfo, StructVariantInfo, TupleVariantInfo, VariantInfo},
    ops::{DynamicEnum, DynamicStruct, DynamicTuple, DynamicVariant},
    registry::TypeRegistry,
};

use super::{
    DeserializerProcessor, InternalDeserializer,
    struct_like_utils::{visit_struct, visit_struct_seq},
    tuple_like_utils::{TupleLikeInfo, visit_tuple},
};

/// A [`Visitor`] for deserializing [`Enum`] values.
///
/// [`Enum`]: crate::Enum
pub(super) struct EnumVisitor<'a, P: DeserializerProcessor> {
    pub enum_info: &'static EnumInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for EnumVisitor<'_, P> {
    type Value = DynamicEnum;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected enum value")
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let mut dynamic_enum = DynamicEnum::default();
        let (variant_info, variant) = data.variant_seed(VariantDeserializer {
            enum_info: self.enum_info,
        })?;

        let value: DynamicVariant = match variant_info {
            VariantInfo::Unit(_) => variant.unit_variant()?.into(),
            VariantInfo::Struct(struct_info) => variant
                .struct_variant(
                    struct_info.field_names(),
                    StructVariantVisitor {
                        struct_info,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?
                .into(),
            VariantInfo::Tuple(tuple_info) if tuple_info.field_len() == 1 => {
                let field_ty = TupleLikeInfo::field_at(tuple_info, 0)?.ty();
                let Some(type_traits) = self.registry.get(field_ty.id()) else {
                    return Err(Error::custom(format!(
                        "no type_traits found for type `{field_ty:?}`"
                    )));
                };

                let value = variant.newtype_variant_seed(InternalDeserializer::new_internal(
                    type_traits,
                    self.registry,
                    self.processor,
                ))?;
                let mut dynamic_tuple = DynamicTuple::default();
                dynamic_tuple.insert_boxed(value);
                dynamic_tuple.into()
            }
            VariantInfo::Tuple(tuple_info) => variant
                .tuple_variant(
                    tuple_info.field_len(),
                    TupleVariantVisitor {
                        tuple_info,
                        registry: self.registry,
                        processor: self.processor,
                    },
                )?
                .into(),
        };
        let variant_name = variant_info.name();
        let variant_index = self
            .enum_info
            .index_of(variant_name)
            .expect("variant should exist");
        dynamic_enum.set_variant_with_index(variant_index, variant_name, value);
        Ok(dynamic_enum)
    }
}

struct VariantDeserializer {
    enum_info: &'static EnumInfo,
}

impl<'de> DeserializeSeed<'de> for VariantDeserializer {
    type Value = &'static VariantInfo;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct VariantVisitor(&'static EnumInfo);

        impl<'de> Visitor<'de> for VariantVisitor {
            type Value = &'static VariantInfo;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("expected either a variant index or variant name")
            }

            fn visit_u32<E>(self, variant_index: u32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.0.variant_at(variant_index as usize).ok_or_else(|| {
                    Error::custom(format!(
                        "no variant found at index `{}` on enum `{}`",
                        variant_index,
                        self.0.type_path()
                    ))
                })
            }

            fn visit_str<E>(self, variant_name: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.0.variant(variant_name).ok_or_else(|| {
                    Error::custom(format!(
                        "no variant found with name `{}` on enum `{}`",
                        variant_name,
                        self.0.type_path()
                    ))
                })
            }
        }

        deserializer.deserialize_identifier(VariantVisitor(self.enum_info))
    }
}

struct StructVariantVisitor<'a, P: DeserializerProcessor> {
    struct_info: &'static StructVariantInfo,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for StructVariantVisitor<'_, P> {
    type Value = DynamicStruct;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected struct variant value")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        visit_struct_seq(&mut seq, self.struct_info, self.registry, self.processor)
    }

    fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        visit_struct(&mut map, self.struct_info, self.registry, self.processor)
    }
}

struct TupleVariantVisitor<'a, P: DeserializerProcessor> {
    tuple_info: &'static TupleVariantInfo,
    registry: &'a TypeRegistry,
    processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for TupleVariantVisitor<'_, P> {
    type Value = DynamicTuple;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected tuple variant value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        visit_tuple(&mut seq, self.tuple_info, self.registry, self.processor)
    }
}
