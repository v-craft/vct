use alloc::format;
use core::{fmt, fmt::Formatter};
use serde::de::{DeserializeSeed, Error, SeqAccess, Visitor};

use crate::{
    info::TupleStructInfo,
    ops::DynamicTupleStruct,
    registry::{TypeRegistry, TypeTraitDefault},
    serde::SkipSerde,
};

use super::{DeserializerProcessor, InternalDeserializer, tuple_like_utils::visit_tuple};

/// A [`Visitor`] for deserializing [`TupleStruct`] values.
///
/// [`TupleStruct`]: crate::TupleStruct
pub(super) struct TupleStructVisitor<'a, P: DeserializerProcessor> {
    pub tuple_struct_info: &'static TupleStructInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for TupleStructVisitor<'_, P> {
    type Value = DynamicTupleStruct;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected tuple struct value")
    }

    fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        visit_tuple(
            &mut seq,
            self.tuple_struct_info,
            self.registry,
            self.processor,
        )
        .map(DynamicTupleStruct::from)
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut dynamic_tuple = DynamicTupleStruct::new();

        let field_info = self
            .tuple_struct_info
            .field_at(0)
            .ok_or(Error::custom("Field at index 0 not found"))?;

        let Some(type_traits) = self.registry.get(field_info.type_id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{:?}`",
                field_info.type_path()
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
                    dynamic_tuple.insert_boxed(val);
                } else {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not support `reflect_clone`.",
                        field_info.type_path(),
                    )));
                }
            } else {
                if let Some(default_value) = type_traits.get::<TypeTraitDefault>() {
                    dynamic_tuple.insert_boxed(default_value.default());
                } else {
                    return Err(Error::custom(
                        "Field `0` skipped serde, but no default value and not support `TypeTraitDefault`.",
                    ));
                }
            }
            return Ok(dynamic_tuple);
        }

        let de = InternalDeserializer::new_internal(type_traits, self.registry, self.processor);
        let value = de.deserialize(deserializer)?;

        dynamic_tuple.insert_boxed(value);

        Ok(dynamic_tuple)
    }
}
