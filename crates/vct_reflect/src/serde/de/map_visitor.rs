use alloc::format;
use core::{fmt, fmt::Formatter};
use serde::de::{Error, MapAccess, Visitor};

use crate::{
    info::MapInfo,
    ops::{DynamicMap, Map},
    registry::TypeRegistry,
};

use super::{DeserializerProcessor, InternalDeserializer};

/// A [`Visitor`] for deserializing [`Map`] values.
///
/// [`Map`]: crate::ops::Map
pub(super) struct MapVisitor<'a, P: DeserializerProcessor> {
    pub map_info: &'static MapInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for MapVisitor<'_, P> {
    type Value = DynamicMap;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected map value")
    }

    fn visit_map<V>(mut self, mut map: V) -> Result<Self::Value, V::Error>
    where
        V: MapAccess<'de>,
    {
        let mut dynamic_map = DynamicMap::new();

        let key_ty = self.map_info.key_ty();
        let Some(key_traits) = self.registry.get(key_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{key_ty:?}`"
            )));
        };

        let value_ty = self.map_info.value_ty();
        let Some(value_traits) = self.registry.get(value_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{value_ty:?}`"
            )));
        };

        while let Some(key) = map.next_key_seed(InternalDeserializer::new_internal(
            key_traits,
            self.registry,
            self.processor.as_deref_mut(),
        ))? {
            let value = map.next_value_seed(InternalDeserializer::new_internal(
                value_traits,
                self.registry,
                self.processor.as_deref_mut(),
            ))?;

            dynamic_map.insert_boxed(key, value);
        }

        Ok(dynamic_map)
    }
}
