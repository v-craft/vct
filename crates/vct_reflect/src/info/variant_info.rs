use crate::info::{
    CustomAttributes, NamedField, UnnamedField,
    attributes::{impl_custom_attributes_fn, impl_with_custom_attributes},
    docs_macro::impl_docs_fn,
};
use alloc::boxed::Box;
use core::{error, fmt};
use vct_os::sync::Arc;
use vct_utils::collections::HashMap;

/// Describes the form of an enum variant.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum VariantKind {
    /// # Example
    ///
    /// ```ignore
    /// enum MyEnum {
    ///   A {   // <--
    ///     foo: usize
    ///   }
    /// }
    /// ```
    Struct,
    /// # Example
    ///
    /// ```ignore
    /// enum MyEnum {
    ///   A(usize) // <--
    /// }
    /// ```
    Tuple,
    /// # Example
    ///
    /// ```ignore
    /// enum MyEnum {
    ///   A // <--
    /// }
    /// ```
    Unit,
}

/// Type info for struct variants.
///
/// ```ignore
/// enum MyEnum {
///   A {  // <--
///     foo: usize
///   }
/// }
/// ```
#[derive(Clone, Debug)]
pub struct StructVariantInfo {
    fields: Box<[NamedField]>,
    field_names: Box<[&'static str]>,
    field_indices: HashMap<&'static str, usize>,
    name: &'static str,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl StructVariantInfo {
    impl_docs_fn!(docs);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new [`StructVariantInfo`].
    pub fn new(name: &'static str, fields: &[NamedField]) -> Self {
        let field_indices = fields
            .iter()
            .enumerate()
            .map(|(index, field)| (field.name(), index))
            .collect();

        let field_names = fields.iter().map(NamedField::name).collect();

        Self {
            name,
            fields: fields.to_vec().into_boxed_slice(),
            field_names,
            field_indices,
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// The name of this variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// A slice containing the names of all fields in order.
    #[inline]
    pub fn field_names(&self) -> &[&'static str] {
        &self.field_names
    }

    /// Get the field with the given name.
    #[inline]
    pub fn field(&self, name: &str) -> Option<&NamedField> {
        self.field_indices
            .get(name)
            .map(|index| &self.fields[*index])
    }

    /// Get the field at the given index.
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&NamedField> {
        self.fields.get(index)
    }

    /// Get the index of the field with the given name.
    #[inline]
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.field_indices.get(name).copied()
    }

    /// Iterate over the fields of this variant.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, NamedField> {
        self.fields.iter()
    }

    /// The total number of fields in this variant.
    #[inline]
    pub fn field_len(&self) -> usize {
        self.fields.len()
    }
}

/// Type info for tuple variants.
///
/// ```ignore
/// enum MyEnum {
///   A(usize) // <--
/// }
/// ```
#[derive(Clone, Debug)]
pub struct TupleVariantInfo {
    fields: Box<[UnnamedField]>,
    name: &'static str,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl TupleVariantInfo {
    impl_docs_fn!(docs);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new [`TupleVariantInfo`].
    pub fn new(name: &'static str, fields: &[UnnamedField]) -> Self {
        // Not inline: Consistent with StructVariantInfo
        Self {
            name,
            fields: fields.to_vec().into_boxed_slice(),
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// The name of this variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get the field at the given index.
    #[inline]
    pub fn field_at(&self, index: usize) -> Option<&UnnamedField> {
        self.fields.get(index)
    }

    /// Iterate over the fields of this variant.
    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, UnnamedField> {
        self.fields.iter()
    }

    /// The total number of fields in this variant.
    #[inline]
    pub fn field_len(&self) -> usize {
        self.fields.len()
    }
}

/// Type info for unit variants.
///
/// ```ignore
/// enum MyEnum {
///   A // <--
/// }
/// ```
#[derive(Clone, Debug)]
pub struct UnitVariantInfo {
    name: &'static str,
    custom_attributes: Arc<CustomAttributes>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl UnitVariantInfo {
    impl_docs_fn!(docs);
    impl_custom_attributes_fn!(custom_attributes);
    impl_with_custom_attributes!(custom_attributes);

    /// Create a new [`UnitVariantInfo`].
    #[inline]
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            custom_attributes: Arc::new(CustomAttributes::default()),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// The name of this variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// A [`VariantInfo`]-specific error.
#[derive(Debug)]
pub struct VariantKindError {
    /// Expected variant type.
    expected: VariantKind,
    /// Received variant type.
    received: VariantKind,
}

impl fmt::Display for VariantKindError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "variant type mismatch: expected {:?}, received {:?}",
            self.expected, self.received
        )
    }
}

impl error::Error for VariantKindError {}

/// Container for compile-time enum variant info.
#[derive(Clone, Debug)]
pub enum VariantInfo {
    /// See [`StructVariantInfo`]
    Struct(StructVariantInfo),
    /// See [`TupleVariantInfo`]
    Tuple(TupleVariantInfo),
    /// See [`UnitVariantInfo`]
    Unit(UnitVariantInfo),
}

macro_rules! impl_cast_fn {
    ($name:ident : $kind:ident => $info:ident) => {
        #[inline]
        pub fn $name(&self) -> Result<&$info, VariantKindError> {
            match self {
                Self::$kind(info) => Ok(info),
                _ => Err(VariantKindError {
                    expected: VariantKind::$kind,
                    received: self.variant_type(),
                }),
            }
        }
    };
}

impl VariantInfo {
    impl_cast_fn!(as_struct_variant: Struct => StructVariantInfo);
    impl_cast_fn!(as_tuple_variant: Tuple => TupleVariantInfo);
    impl_cast_fn!(as_unit_variant: Unit => UnitVariantInfo);

    impl_custom_attributes_fn!(self => match self {
        Self::Struct(info) => info.custom_attributes(),
        Self::Tuple(info) => info.custom_attributes(),
        Self::Unit(info) => info.custom_attributes(),
    });

    /// The name of the enum variant.
    #[inline]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Struct(info) => info.name(),
            Self::Tuple(info) => info.name(),
            Self::Unit(info) => info.name(),
        }
    }

    /// Returns the [kind] of this variant.
    ///
    /// [kind]: VariantKind
    #[inline]
    pub fn variant_type(&self) -> VariantKind {
        match self {
            Self::Struct(_) => VariantKind::Struct,
            Self::Tuple(_) => VariantKind::Tuple,
            Self::Unit(_) => VariantKind::Unit,
        }
    }

    /// The docstring of the underlying variant, if any.
    #[cfg(feature = "reflect_docs")]
    #[inline]
    pub fn docs(&self) -> Option<&str> {
        match self {
            Self::Struct(info) => info.docs(),
            Self::Tuple(info) => info.docs(),
            Self::Unit(info) => info.docs(),
        }
    }
}
