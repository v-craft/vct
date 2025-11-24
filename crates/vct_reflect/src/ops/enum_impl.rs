use core::{
    any::TypeId, fmt, hash::{Hash, Hasher}
};
use alloc::{
    format,
    borrow::{Cow, ToOwned},
    boxed::Box,
    string::String,
};
use crate::{
    PartialReflect, Reflect, info::{
        EnumInfo, MaybeTyped, ReflectKind, TypeInfo, TypePath, VariantType
    }, ops::{
        ApplyError, DynamicStruct, DynamicTuple, DynamicVariant, ReflectMut, ReflectOwned, ReflectRef, Struct, Tuple, VariantFieldIter
    }, reflect_hasher
};

#[derive(Default)]
pub struct DynamicEnum {
    target_type: Option<&'static TypeInfo>,
    variant_index: usize,
    variant_name: Cow<'static, str>,
    variant: DynamicVariant,
}

impl TypePath for DynamicEnum {
    fn type_path() -> &'static str {
        "vct_reflect::ops::DynamicEnum"
    }
    fn short_type_path() -> &'static str {
        "DynamicEnum"
    }
    fn type_ident() -> Option<&'static str> {
        Some("DynamicEnum")
    }
    fn crate_name() -> Option<&'static str> {
        Some("vct_reflect")
    }
    fn module_path() -> Option<&'static str> {
        Some("vct_reflect::ops")
    }
}

impl DynamicEnum {
    #[inline]
    pub fn new<I: Into<Cow<'static, str>>, V: Into<DynamicVariant>>(variant_name: I, variant: V) -> Self {
        Self {
            target_type: None,
            variant_index: 0,
            variant_name: variant_name.into(),
            variant: variant.into(),
        }
    }

    #[inline]
    pub fn new_with_index<I: Into<Cow<'static, str>>, V: Into<DynamicVariant>>(
        variant_index: usize,
        variant_name: I,
        variant: V,
    ) -> Self {
        Self {
            target_type: None,
            variant_index,
            variant_name: variant_name.into(),
            variant: variant.into(),
        }
    }

    pub fn set_target_type_info(&mut self, target_type: Option<&'static TypeInfo>) {
        if let Some(target_type) = target_type {
            assert!(
                matches!(target_type, TypeInfo::Enum(_)),
                "expected TypeInfo::Enum but received: {target_type:?}",
            );
        }

        self.target_type = target_type;
    }


    #[inline]
    pub fn set_variant<I: Into<Cow<'static, str>>, V: Into<DynamicVariant>>(&mut self, name: I, variant: V) {
        self.variant_name = name.into();
        self.variant = variant.into();
    }

    #[inline]
    pub fn set_variant_with_index<I: Into<Cow<'static, str>>, V: Into<DynamicVariant>>(
        &mut self,
        variant_index: usize,
        variant_name: I,
        variant: V,
    ) {
        self.variant_index = variant_index;
        self.variant_name = variant_name.into();
        self.variant = variant.into();
    }

    #[inline]
    pub fn variant(&self) -> &DynamicVariant {
        &self.variant
    }

    #[inline]
    pub fn variant_mut(&mut self) -> &mut DynamicVariant {
        &mut self.variant
    }

    #[inline]
    pub fn from<TEnum: Enum>(value: TEnum) -> Self {
        // copy value instead of referencing
        Self::from_ref(&value)
    }

    pub fn from_ref<TEnum: Enum + ?Sized>(value: &TEnum) -> Self {
        let type_info = value.get_target_type_info();
        let mut dyn_enum = match value.variant_type() {
            VariantType::Unit => DynamicEnum::new_with_index(
                value.variant_index(),
                value.variant_name().to_owned(),
                DynamicVariant::Unit,
            ),
            VariantType::Tuple => {
                let mut data = DynamicTuple::default();
                for field in value.iter_fields() {
                    data.insert_boxed(field.value().to_dynamic());
                }
                DynamicEnum::new_with_index(
                    value.variant_index(),
                    value.variant_name().to_owned(),
                    DynamicVariant::Tuple(data),
                )
            }
            VariantType::Struct => {
                let mut data = DynamicStruct::default();
                for field in value.iter_fields() {
                    let name = field.name().unwrap();
                    data.insert_boxed(name.to_owned(), field.value().to_dynamic());
                }
                DynamicEnum::new_with_index(
                    value.variant_index(),
                    value.variant_name().to_owned(),
                    DynamicVariant::Struct(data),
                )
            }
        };

        dyn_enum.set_target_type_info(type_info);
        dyn_enum
    }
}

impl PartialReflect for DynamicEnum {
    #[inline]
    fn get_target_type_info(&self) -> Option<&'static TypeInfo> {
        self.target_type
    }

    #[inline]
    fn as_partial_reflect(&self) -> &dyn PartialReflect {
        self
    }

    #[inline]
    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect {
        self
    }

    #[inline]
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect> {
        self
    }

    #[inline]
    fn try_as_reflect(&self) -> Option<&dyn Reflect> {
        None
    }

    #[inline]
    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect> {
        None
    }

    #[inline]
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>> {
        Err(self)
    }

    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError> {
        let other = value.reflect_ref().as_enum()?;
        if self.variant_name() == other.variant_name() {
            match other.variant_type() {
                VariantType::Struct => {
                    for other_field in other.iter_fields() {
                        let name = other_field.name().unwrap();
                        if let Some(field) = self.field_mut(name) {
                            field.try_apply(other_field.value())?;
                        }
                    }
                },
                VariantType::Tuple => {
                    for (index, other_field) in other.iter_fields().enumerate() {
                        if let Some(field) = self.field_at_mut(index) {
                            field.try_apply(other_field.value())?;
                        }
                    }
                },
                VariantType::Unit => {},
            }
        } else {
            let dyn_variant = match other.variant_type() {
                VariantType::Unit => DynamicVariant::Unit,
                VariantType::Tuple => {
                    let mut dyn_tuple = DynamicTuple::default();
                    for other_field in other.iter_fields() {
                        dyn_tuple.insert_boxed(other_field.value().to_dynamic());
                    }
                    DynamicVariant::Tuple(dyn_tuple)
                },
                VariantType::Struct => {
                    let mut dyn_struct = DynamicStruct::default();
                    for other_field in other.iter_fields() {
                        dyn_struct.insert_boxed(
                            other_field.name().unwrap().to_owned(),
                            other_field.value().to_dynamic(),
                        );
                    }
                    DynamicVariant::Struct(dyn_struct)
                }
            };
            self.set_variant(other.variant_name().to_owned(), dyn_variant);
        }
        Ok(())
    }

    #[inline]
    fn reflect_kind(&self) -> ReflectKind {
        ReflectKind::Enum
    }

    #[inline]
    fn reflect_ref(&self) -> ReflectRef<'_> {
        ReflectRef::Enum(self)
    }

    #[inline]
    fn reflect_mut(&mut self) -> ReflectMut<'_> {
        ReflectMut::Enum(self)
    }

    #[inline]
    fn reflect_owned(self: Box<Self>) -> ReflectOwned {
        ReflectOwned::Enum(self)
    }

    fn reflect_hash(&self) -> Option<u64> {
        let mut hasher = reflect_hasher();
        TypeId::of::<Self>().hash(&mut hasher);

        self.variant_name().hash(&mut hasher);
        self.variant_type().hash(&mut hasher);
        for field in self.iter_fields() {
            hasher.write_u64(field.value().reflect_hash()?);
        }
        Some(hasher.finish())
    }

    fn reflect_partial_eq(&self, other: &dyn PartialReflect) -> Option<bool> {
        let ReflectRef::Enum(other) = other.reflect_ref() else {
            return Some(false);
        };

        if self.variant_name() != other.variant_name() {
            return Some(false);
        }

        if self.variant_type() != other.variant_type() {
            return Some(false);
        }

        match self.variant_type() {
            VariantType::Unit => Some(true),
            VariantType::Tuple => {
                for (idx, field) in self.iter_fields().enumerate() {
                    if let Some(other_field) = other.field_at(idx) {
                        let result = field.value().reflect_partial_eq(other_field);
                        if result != Some(true) {
                            return Some(false);
                        }
                    } else {
                        return Some(false);
                    }
                }
                Some(true)
            },
            VariantType::Struct => {
                for field in self.iter_fields() {
                    if let Some(other_field) = other.field(field.name().unwrap()) {
                        let result = field.value().reflect_partial_eq(other_field);
                        if result != Some(true) {
                            return Some(false);
                        }
                    } else {
                        return Some(false);
                    }
                }
                Some(true)
            }
        }
    }

    #[inline]
    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DynamicEnum(")?;
        enum_debug(self, f)?;
        write!(f, ")")
    }

    #[inline]
    fn is_dynamic(&self) -> bool {
        true
    }
}

impl MaybeTyped for DynamicEnum {}

impl fmt::Debug for DynamicEnum {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

pub trait Enum: PartialReflect {
    fn field(&self, name: &str) -> Option<&dyn PartialReflect>;

    fn field_at(&self, index: usize) -> Option<&dyn PartialReflect>;

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect>;

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect>;

    fn index_of(&self, name: &str) -> Option<usize>;

    fn name_at(&self, index: usize) -> Option<&str>;

    fn iter_fields(&self) -> VariantFieldIter<'_>;

    fn field_len(&self) -> usize;

    fn variant_name(&self) -> &str;

    fn variant_index(&self) -> usize;

    fn variant_type(&self) -> VariantType;

    fn to_dynamic_enum(&self) -> DynamicEnum {
        DynamicEnum::from_ref(self)
    }

    fn is_variant(&self, variant_type: VariantType) -> bool {
        self.variant_type() == variant_type
    }

    fn variant_path(&self) -> String {
        format!("{}::{}", self.reflect_type_path(), self.variant_name())
    }

    fn get_target_enum_info(&self) -> Option<&'static EnumInfo> {
        self.get_target_type_info()?.as_enum().ok()
    }
}

impl Enum for DynamicEnum {
    fn field(&self, name: &str) -> Option<&dyn PartialReflect> {
        if let DynamicVariant::Struct(data) = &self.variant {
            data.field(name)
        } else {
            None
        }
    }

    fn field_at(&self, index: usize) -> Option<&dyn PartialReflect> {
        match &self.variant {
            DynamicVariant::Tuple(data) => data.field(index),
            DynamicVariant::Struct(data) => data.field_at(index),
            DynamicVariant::Unit => None,
        }
    }

    fn field_mut(&mut self, name: &str) -> Option<&mut dyn PartialReflect> {
        if let DynamicVariant::Struct(data) = &mut self.variant {
            data.field_mut(name)
        } else {
            None
        }
    }

    fn field_at_mut(&mut self, index: usize) -> Option<&mut dyn PartialReflect> {
        match &mut self.variant {
            DynamicVariant::Tuple(data) => data.field_mut(index),
            DynamicVariant::Struct(data) => data.field_at_mut(index),
            DynamicVariant::Unit => None,
        }
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        if let DynamicVariant::Struct(data) = &self.variant {
            data.index_of(name)
        } else {
            None
        }
    }

    fn name_at(&self, index: usize) -> Option<&str> {
        if let DynamicVariant::Struct(data) = &self.variant {
            data.name_at(index)
        } else {
            None
        }
    }

    #[inline]
    fn iter_fields(&self) -> VariantFieldIter<'_> {
        VariantFieldIter::new(self)
    }

    fn field_len(&self) -> usize {
        match &self.variant {
            DynamicVariant::Unit => 0,
            DynamicVariant::Tuple(data) => data.field_len(),
            DynamicVariant::Struct(data) => data.field_len(),
        }
    }

    #[inline]
    fn variant_name(&self) -> &str {
        &self.variant_name
    }

    #[inline]
    fn variant_index(&self) -> usize {
        self.variant_index
    }

    #[inline]
    fn variant_type(&self) -> VariantType {
        match &self.variant {
            DynamicVariant::Unit => VariantType::Unit,
            DynamicVariant::Tuple(..) => VariantType::Tuple,
            DynamicVariant::Struct(..) => VariantType::Struct,
        }
    }
}

pub fn enum_partial_eq<TEnum: Enum + ?Sized>(x: &TEnum, y: &dyn PartialReflect) -> Option<bool> {
    let ReflectRef::Enum(y) = y.reflect_ref() else {
        return Some(false);
    };

    if x.variant_name() != y.variant_name() {
        return Some(false);
    }

    if x.variant_type() != y.variant_type() {
        return Some(false);
    }

    match x.variant_type() {
        VariantType::Unit => Some(true),
        VariantType::Tuple => {
            for (idx, field) in x.iter_fields().enumerate() {
                if let Some(y_field) = y.field_at(idx) {
                    let result = field.value().reflect_partial_eq(y_field);
                    if result != Some(true) {
                        return Some(false);
                    }
                } else {
                    return Some(false);
                }
            }
            Some(true)
        },
        VariantType::Struct => {
            for field in x.iter_fields() {
                if let Some(y_field) = y.field(field.name().unwrap()) {
                    let result = field.value().reflect_partial_eq(y_field);
                    if result != Some(true) {
                        return Some(false);
                    }
                } else {
                    return Some(false);
                }
            }
            Some(true)
        }
    }
}

pub fn enum_debug(dyn_enum: &dyn Enum, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match dyn_enum.variant_type() {
        VariantType::Unit => f.write_str(dyn_enum.variant_name()),
        VariantType::Tuple => {
            let mut debug = f.debug_tuple(dyn_enum.variant_name());
            for field in dyn_enum.iter_fields() {
                debug.field(&field.value() as &dyn fmt::Debug);
            }
            debug.finish()
        }
        VariantType::Struct => {
            let mut debug = f.debug_struct(dyn_enum.variant_name());
            for field in dyn_enum.iter_fields() {
                debug.field(field.name().unwrap(), &field.value() as &dyn fmt::Debug);
            }
            debug.finish()
        }
    }
}

