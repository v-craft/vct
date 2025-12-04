use alloc::{boxed::Box, format, string::ToString, vec::Vec};
use core::{fmt, fmt::Formatter};
use serde::de::{Error, SeqAccess, Visitor};

use crate::{Reflect, info::ArrayInfo, ops::DynamicArray, registry::TypeRegistry};

use super::{DeserializerProcessor, InternalDeserializer};

/// A [`Visitor`] for deserializing [`Array`] values.
///
/// [`Array`]: crate::ops::Array
pub(super) struct ArrayVisitor<'a, P: DeserializerProcessor> {
    pub array_info: &'static ArrayInfo,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a mut P>,
}

impl<'de, P: DeserializerProcessor> Visitor<'de> for ArrayVisitor<'_, P> {
    type Value = DynamicArray;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        formatter.write_str("reflected array value")
    }

    fn visit_seq<V>(mut self, mut seq: V) -> Result<Self::Value, V::Error>
    where
        V: SeqAccess<'de>,
    {
        let item_ty = self.array_info.item_ty();
        let Some(type_traits) = self.registry.get(item_ty.id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{item_ty:?}`"
            )));
        };

        let mut vec: Vec<Box<dyn Reflect>> =
            Vec::with_capacity(seq.size_hint().unwrap_or_default());

        while let Some(value) = seq.next_element_seed(InternalDeserializer::new_internal(
            type_traits,
            self.registry,
            self.processor.as_deref_mut(),
        ))? {
            vec.push(value);
        }

        if vec.len() != self.array_info.capacity() {
            return Err(Error::invalid_length(
                vec.len(),
                &self.array_info.capacity().to_string().as_str(),
            ));
        }

        Ok(DynamicArray::new(vec.into_boxed_slice()))
    }
}
