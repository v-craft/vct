use crate::{
    PartialReflect,
    info::VariantKind,
    ops::{DynamicStruct, DynamicTuple, Enum, Struct, Tuple},
};

#[derive(Default, Debug)] // impl Debug: All fields have already impl Debug
pub enum DynamicVariant {
    #[default]
    Unit,
    Tuple(DynamicTuple),
    Struct(DynamicStruct),
}

impl Clone for DynamicVariant {
    fn clone(&self) -> Self {
        match self {
            Self::Unit => Self::Unit,
            Self::Tuple(data) => Self::Tuple(data.to_dynamic_tuple()),
            Self::Struct(data) => Self::Struct(data.to_dynamic_struct()),
        }
    }
}

impl From<()> for DynamicVariant {
    #[inline]
    fn from(_: ()) -> Self {
        Self::Unit
    }
}

impl From<DynamicTuple> for DynamicVariant {
    #[inline]
    fn from(value: DynamicTuple) -> Self {
        Self::Tuple(value)
    }
}

impl From<DynamicStruct> for DynamicVariant {
    #[inline]
    fn from(value: DynamicStruct) -> Self {
        Self::Struct(value)
    }
}

pub enum VariantField<'a> {
    /// The name and value of a field in a struct variant.
    Struct(&'a str, &'a dyn PartialReflect),
    /// The value of a field in a tuple variant.
    Tuple(&'a dyn PartialReflect),
}

impl<'a> VariantField<'a> {
    /// Returns the name of a struct variant field, or [`None`] for a tuple variant field.
    #[inline]
    pub fn name(&self) -> Option<&'a str> {
        if let Self::Struct(name, ..) = self {
            Some(*name)
        } else {
            None
        }
    }

    /// Gets a reference to the value of this field.
    #[inline]
    pub fn value(&self) -> &'a dyn PartialReflect {
        match *self {
            Self::Struct(_, value) | Self::Tuple(value) => value,
        }
    }
}

/// An iterator over the fields in the current enum variant.
pub struct VariantFieldIter<'a> {
    container: &'a dyn Enum,
    index: usize,
}

impl<'a> VariantFieldIter<'a> {
    #[inline(always)]
    pub fn new(container: &'a dyn Enum) -> Self {
        Self {
            container,
            index: 0,
        }
    }
}

impl<'a> Iterator for VariantFieldIter<'a> {
    type Item = VariantField<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let value = match self.container.variant_type() {
            VariantKind::Unit => None,
            VariantKind::Tuple => Some(VariantField::Tuple(self.container.field_at(self.index)?)),
            VariantKind::Struct => {
                let name = self.container.name_at(self.index)?;
                Some(VariantField::Struct(name, self.container.field(name)?))
            }
        };
        self.index += value.is_some() as usize;
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.container.field_len();
        (size - self.index, Some(size))
    }
}

impl<'a> ExactSizeIterator for VariantFieldIter<'a> {}
