use alloc::format;
use core::{fmt, fmt::Formatter};
use serde::de::{Error, SeqAccess, Visitor};

use crate::{
    info::SetInfo,
    ops::{DynamicSet, Set},
    registry::TypeRegistry,
};

use super::{DeserializerProcessor, InternalDeserializer};

/// A [`Visitor`] for deserializing [`Set`] values.
///
/// [`Set`]: crate::ops::Set
pub(super) struct SetVisitor<'a, P: DeserializerProcessor> {
    pub set_info: &'static SetInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for SetVisitor<'_, P> {
    type Value = DynamicSet;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected set value")
    }

    fn visit_seq<V>(mut self, mut set: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let mut dynamic_set = DynamicSet::new();

        let value_ty = self.set_info.value_ty();
        let Some(type_traits) = self.registry.get(value_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{value_ty:?}`"
            )));
        };

        while let Some(value) = set.next_element_seed(InternalDeserializer::new_internal(
            type_traits,
            self.registry,
            self.processor.as_deref_mut(),
        ))? {
            dynamic_set.insert_boxed(value);
        }

        Ok(dynamic_set)
    }
}
