use alloc::format;
use core::fmt;
use serde::de::{DeserializeSeed, Error, Visitor};

use crate::{
    info::{EnumInfo, VariantInfo},
    ops::{DynamicEnum, DynamicTuple},
    registry::TypeRegistry,
};

use super::{DeserializerProcessor, InternalDeserializer};

/// A [`Visitor`] for deserializing [`Option`] values.
pub(super) struct OptionVisitor<'a, P: DeserializerProcessor> {
    pub enum_info: &'static EnumInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for OptionVisitor<'_, P> {
    type Value = DynamicEnum;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("reflected option value of type ")?;
        formatter.write_str(self.enum_info.type_path())
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let mut option = DynamicEnum::default();
        option.set_variant("None", ());
        Ok(option)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let Some(variant_info) = self.enum_info.variant("Some") else {
            return Err(Error::custom(format!(
                "invalid variant, expected `Some(_)` but got: {:?}",
                self.enum_info
            )));
        };

        match variant_info {
            VariantInfo::Tuple(tuple_info) if tuple_info.field_len() == 1 => {
                let field_ty = match tuple_info.field_at(0) {
                    Some(field) => field.ty(),
                    None => {
                        return Err(Error::custom(format!(
                            "invalid variant, expected `Some(_)` but got: {tuple_info:?}"
                        )));
                    }
                };

                let Some(type_traits) = self.registry.get(field_ty.id()) else {
                    return Err(Error::custom(format!(
                        "no type_traits found for type `{field_ty:?}`"
                    )));
                };

                let de =
                    InternalDeserializer::new_internal(type_traits, self.registry, self.processor);
                let mut value = DynamicTuple::new();
                value.insert_boxed(de.deserialize(deserializer)?);
                let mut option = DynamicEnum::default();
                option.set_variant("Some", value);
                Ok(option)
            }
            info => Err(Error::custom(format!(
                "invalid variant, expected `Some(_)` but got: {info:?}"
            ))),
        }
    }
}
