use alloc::{format, string::ToString};
use serde::de::{Error, SeqAccess};

use crate::{
    info::{TupleInfo, TupleStructInfo, TupleVariantInfo, UnnamedField},
    ops::DynamicTuple,
    registry::{TypeRegistry, TypeTraitDefault},
    serde::SkipSerde,
};

use super::{DeserializerProcessor, InternalDeserializer};

pub(super) trait TupleLikeInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E>;
    fn field_len(&self) -> usize;
}

impl TupleLikeInfo for TupleInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on tuple `{}`",
                index,
                self.type_path(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }
}

impl TupleLikeInfo for TupleStructInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on tuple struct `{}`",
                index,
                self.type_path(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }
}

impl TupleLikeInfo for TupleVariantInfo {
    fn field_at<E: Error>(&self, index: usize) -> Result<&UnnamedField, E> {
        Self::field_at(self, index).ok_or_else(|| {
            Error::custom(format!(
                "no field at index `{}` on tuple variant `{}`",
                index,
                self.name(),
            ))
        })
    }

    #[inline]
    fn field_len(&self) -> usize {
        Self::field_len(self)
    }
}

/// Deserializes a [tuple-like] type from a sequence of elements, returning a [`DynamicTuple`].
///
/// [tuple-like]: TupleLikeInfo
pub(super) fn visit_tuple<'de, T, V, P>(
    seq: &mut V,
    info: &T,
    registry: &TypeRegistry,
    mut processor: Option<&mut P>,
) -> Result<DynamicTuple, V::Error>
where
    T: TupleLikeInfo,
    V: SeqAccess<'de>,
    P: DeserializerProcessor,
{
    let mut dynamic_tuple = DynamicTuple::new();

    let len = info.field_len();

    for index in 0..len {
        let field_info = info.field_at::<V::Error>(index)?;

        let Some(type_traits) = registry.get(field_info.type_id()) else {
            return Err(Error::custom(format!(
                "no type_traits found for type `{:?}`",
                field_info.ty()
            )));
        };

        // skip serde fields
        if let Some(skip_serde) = field_info.get_attribute::<SkipSerde>() {
            if let Some(default_value) = &skip_serde.0 {
                if default_value.type_id() != field_info.type_id() {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not match the actual type `{}`.",
                        default_value.reflect_type_path(),
                        field_info.type_path(),
                    )));
                }
                if let Ok(val) = default_value.reflect_clone() {
                    dynamic_tuple.insert_boxed(val);
                } else {
                    return Err(Error::custom(format!(
                        "The type of the default value (`{}`) in the custom attribute does not support `reflect_clone`.",
                        field_info.type_path(),
                    )));
                }
            } else {
                if let Some(default_value) = type_traits.get::<TypeTraitDefault>() {
                    dynamic_tuple.insert_boxed(default_value.default());
                } else {
                    return Err(Error::custom(format!(
                        "Field `{index}` skipped serde, but no default value and not support `TypeTraitDefault`.",
                    )));
                }
            }
            continue;
        }

        let value = seq
            .next_element_seed(InternalDeserializer::new_internal(
                type_traits,
                registry,
                processor.as_deref_mut(),
            ))?
            .ok_or_else(|| Error::invalid_length(index, &len.to_string().as_str()))?;
        dynamic_tuple.insert_boxed(value);
    }

    Ok(dynamic_tuple)
}
