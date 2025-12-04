use serde::{Serialize, ser::SerializeSeq};

use super::{InternalSerializer, SerializerProcessor};
use crate::{ops::Set, registry::TypeRegistry};

/// A serializer for [`Set`] values.
pub(super) struct SetSerializer<'a, P: SerializerProcessor> {
    pub set: &'a dyn Set,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for SetSerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_seq(Some(self.set.len()))?;
        for value in self.set.iter() {
            state.serialize_element(&InternalSerializer::new_internal(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
