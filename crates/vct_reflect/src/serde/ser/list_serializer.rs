use serde::{Serialize, ser::SerializeSeq};

use super::{InternalSerializer, SerializerProcessor};
use crate::{ops::List, registry::TypeRegistry};

/// A serializer for [`List`] values.
pub(super) struct ListSerializer<'a, P: SerializerProcessor> {
    pub list: &'a dyn List,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for ListSerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_seq(Some(self.list.len()))?;
        for value in self.list.iter() {
            state.serialize_element(&InternalSerializer::new_internal(
                value,
                self.registry,
                self.processor,
            ))?;
        }
        state.end()
    }
}
