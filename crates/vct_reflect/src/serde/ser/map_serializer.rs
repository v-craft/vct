use serde::{Serialize, ser::SerializeMap};

use super::{InternalSerializer, SerializerProcessor};
use crate::{ops::Map, registry::TypeRegistry};

/// A serializer for [`Map`] values.
pub(super) struct MapSerializer<'a, P: SerializerProcessor> {
    pub map: &'a dyn Map,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for MapSerializer<'_, P> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_map(Some(self.map.len()))?;
        for (key, value) in self.map.iter() {
            state.serialize_entry(
                &InternalSerializer::new_internal(key, self.registry, self.processor),
                &InternalSerializer::new_internal(value, self.registry, self.processor),
            )?;
        }
        state.end()
    }
}
