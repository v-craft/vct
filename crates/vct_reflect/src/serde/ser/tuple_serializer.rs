use serde::{Serialize, ser::SerializeTuple};

use super::{InternalSerializer, SerializerProcessor};
use crate::{ops::Tuple, registry::TypeRegistry};

/// A serializer for [`Tuple`] values.
pub(super) struct TupleSerializer<'a, P: SerializerProcessor> {
    pub tuple: &'a dyn Tuple,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for TupleSerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_tuple(self.tuple.field_len())?;

        for value in self.tuple.iter_fields() {
            state.serialize_element(&InternalSerializer::new_internal(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
