use alloc::format;
use serde::{
    Serialize,
    ser::{Error, SerializeTupleStruct},
};

use super::{InternalSerializer, SerializerProcessor};
use crate::{info::TypeInfo, ops::TupleStruct, registry::TypeRegistry, serde::SkipSerde};

/// A serializer for [`TupleStruct`] values.
pub(super) struct TupleStructSerializer<'a, P: SerializerProcessor> {
    pub tuple_struct: &'a dyn TupleStruct,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for TupleStructSerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let type_info = match self.tuple_struct.represented_type_info() {
            Some(info) => info,
            None => {
                return Err(Error::custom(format! {
                    "cannot get represented type info for `{}`",
                    self.tuple_struct.reflect_type_path()
                }));
            }
        };

        let tuple_struct_info = match type_info {
            TypeInfo::TupleStruct(tuple_struct_info) => tuple_struct_info,
            info => {
                return Err(Error::custom(format!(
                    "expected tuple struct but received {info:?}"
                )));
            }
        };

        let field_len = tuple_struct_info
            .iter()
            .map(|f| !f.has_attribute::<SkipSerde>() as usize)
            .sum::<usize>();

        let mut state = serializer.serialize_tuple_struct(
            tuple_struct_info
                .type_path_table()
                .ident()
                .unwrap_or(crate::serde::NO_IDENT),
            field_len,
        )?;

        for field_info in tuple_struct_info.iter() {
            if field_info.has_attribute::<SkipSerde>() {
                continue;
            }
            let index = field_info.index();
            if let Some(value) = self.tuple_struct.field(index) {
                state.serialize_field(&InternalSerializer::new_internal(
                    value,
                    self.registry,
                    self.processor,
                ))?;
            } else {
                return Err(Error::custom(format!(
                    "field `{index}` was missing while serializing type {}",
                    tuple_struct_info.type_path()
                )));
            }
        }

        state.end()
    }
}
