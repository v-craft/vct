use alloc::format;
use serde::{
    Serialize,
    ser::{Error, SerializeStructVariant, SerializeTupleVariant},
};

use super::{InternalSerializer, SerializerProcessor};
use crate::{
    info::{TypeInfo, VariantInfo, VariantKind},
    ops::Enum,
    registry::TypeRegistry,
};

/// A serializer for [`Enum`] values.
pub(super) struct EnumSerializer<'a, P: SerializerProcessor> {
    pub enum_value: &'a dyn Enum,
    pub registry: &'a TypeRegistry,
    pub processor: Option<&'a P>,
}

impl<P: SerializerProcessor> Serialize for EnumSerializer<'_, P> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let type_info = match self.enum_value.represented_type_info() {
            Some(info) => info,
            None => {
                return Err(Error::custom(format! {
                    "cannot get represented type info for `{}`",
                    self.enum_value.reflect_type_path()
                }));
            }
        };

        let enum_info = match type_info {
            TypeInfo::Enum(enum_info) => enum_info,
            info => {
                return Err(Error::custom(format!(
                    "expected enum but received {info:?}"
                )));
            }
        };

        let enum_name = enum_info
            .type_path_table()
            .ident()
            .unwrap_or(crate::serde::NO_IDENT);
        let variant_index = self.enum_value.variant_index() as u32;
        let variant_info = enum_info
            .variant_at(variant_index as usize)
            .ok_or_else(|| {
                Error::custom(format!("variant at index `{variant_index}` does not exist"))
            })?;
        let variant_name = variant_info.name();
        let variant_kind = self.enum_value.variant_kind();
        let field_len = self.enum_value.field_len();

        match variant_kind {
            VariantKind::Unit => {
                if type_info.type_path_table().module_path() == Some("core::option")
                    && type_info.type_path_table().ident() == Some("Option")
                {
                    serializer.serialize_none()
                } else {
                    serializer.serialize_unit_variant(enum_name, variant_index, variant_name)
                }
            }
            VariantKind::Struct => {
                let struct_info = match variant_info {
                    VariantInfo::Struct(struct_info) => struct_info,
                    info => {
                        return Err(Error::custom(format!(
                            "expected struct variant type but received {info:?}"
                        )));
                    }
                };
                let mut state = serializer.serialize_struct_variant(
                    enum_name,
                    variant_index,
                    variant_name,
                    field_len,
                )?;

                for field_info in struct_info.iter() {
                    let name = field_info.name();
                    if let Some(value) = self.enum_value.field(name) {
                        state.serialize_field(
                            name,
                            &InternalSerializer::new_internal(value, self.registry, self.processor),
                        )?;
                    } else {
                        return Err(Error::custom(format!(
                            "field `{name}` was missing while serializing type {}",
                            enum_info.type_path()
                        )));
                    }
                }

                state.end()
            }
            VariantKind::Tuple if field_len == 1 => {
                let field = self.enum_value.field_at(0).unwrap();

                if type_info.type_path_table().module_path() == Some("core::option")
                    && type_info.type_path_table().ident() == Some("Option")
                {
                    serializer.serialize_some(&InternalSerializer::new_internal(
                        field,
                        self.registry,
                        self.processor,
                    ))
                } else {
                    serializer.serialize_newtype_variant(
                        enum_name,
                        variant_index,
                        variant_name,
                        &InternalSerializer::new_internal(field, self.registry, self.processor),
                    )
                }
            }
            VariantKind::Tuple => {
                let tuple_info = match variant_info {
                    VariantInfo::Tuple(tuple_info) => tuple_info,
                    info => {
                        return Err(Error::custom(format!(
                            "expected tuple variant type but received {info:?}"
                        )));
                    }
                };
                let mut state = serializer.serialize_tuple_variant(
                    enum_name,
                    variant_index,
                    variant_name,
                    field_len,
                )?;

                for field_info in tuple_info.iter() {
                    let index = field_info.index();
                    if let Some(value) = self.enum_value.field_at(index) {
                        state.serialize_field(&InternalSerializer::new_internal(
                            value,
                            self.registry,
                            self.processor,
                        ))?;
                    } else {
                        return Err(Error::custom(format!(
                            "field `{index}` was missing while serializing type {}",
                            enum_info.type_path()
                        )));
                    }
                }

                state.end()
            }
        }
    }
}
