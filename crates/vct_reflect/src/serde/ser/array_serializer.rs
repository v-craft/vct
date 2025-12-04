use serde::{Serialize, ser::SerializeTuple};

use super::{InternalSerializer, SerializerProcessor};
use crate::{ops::Array, registry::TypeRegistry};

/// A serializer for [`Array`] values.
pub(super) struct ArraySerializer<'a, P: SerializerProcessor> {
    pub array: &'a dyn Array,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for ArraySerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_tuple(self.array.len())?;
        for value in self.array.iter() {
            state.serialize_element(&InternalSerializer::new_internal(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
