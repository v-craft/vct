use alloc::format;
use core::{fmt, fmt::Formatter};
use serde::de::{Error, SeqAccess, Visitor};

use crate::{info::ListInfo, ops::DynamicList, registry::TypeRegistry};

use super::{DeserializerProcessor, InternalDeserializer};

/// A [`Visitor`] for deserializing [`List`] values.
///
/// [`List`]: crate::ops::List
pub(super) struct ListVisitor<'a, P: DeserializerProcessor> {
    pub list_info: &'static ListInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for ListVisitor<'_, P> {
    type Value = DynamicList;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected list value")
    }

    fn visit_seq<V>(mut self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let item_ty = self.list_info.item_ty();
        let Some(type_traits) = self.registry.get(item_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{item_ty:?}`"
            )));
        };

        let mut list = DynamicList::with_capacity(seq.size_hint().unwrap_or_default());

        while let Some(value) = seq.next_element_seed(InternalDeserializer::new_internal(
            type_traits,
            self.registry,
            self.processor.as_deref_mut(),
        ))? {
            list.push_box(value);
        }

        Ok(list)
    }
}
