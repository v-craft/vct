use alloc::format;
use serde::{
    Serialize,
    ser::{Error, SerializeStruct},
};

use crate::{info::TypeInfo, ops::Struct, registry::TypeRegistry, serde::SkipSerde};

use super::{InternalSerializer, SerializerProcessor};

/// A serializer for [`Struct`] values.
pub(super) struct StructSerializer<'a, P: SerializerProcessor> {
    pub struct_value: &'a dyn Struct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for StructSerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let type_info = match self.struct_value.represented_type_info() {
            Some(info) => info,
            None => {
                return Err(Error::custom(format! {
                    "cannot get represented type info for `{}`",
                    self.struct_value.reflect_type_path()
                }));
            }
        };

        let struct_info = match type_info {
            TypeInfo::Struct(struct_info) => struct_info,
            info => {
                return Err(Error::custom(format!(
                    "expected struct but received {info:?}"
                )));
            }
        };

        let field_len = struct_info
            .iter()
            .map(|f| !f.has_attribute::<SkipSerde>() as usize)
            .sum::<usize>();

        let mut state = serializer.serialize_struct(
            struct_info
                .type_path_table()
                .ident()
                .unwrap_or(crate::serde::NO_IDENT),
            field_len,
        )?;

        for field_info in struct_info.iter() {
            if field_info.has_attribute::<SkipSerde>() {
                continue;
            }
            let name = field_info.name();
            if let Some(value) = self.struct_value.field(name) {
                state.serialize_field(
                    name,
                    &InternalSerializer::new_internal(value, self.registry, self.processor),
                )?;
            } else {
                return Err(Error::custom(format!(
                    "field `{name}` was missing while serializing type {}",
                    struct_info.type_path()
                )));
            }
        }

        state.end()
    }
}
