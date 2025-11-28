// A macro used to simplify code
mod docs_macro;

mod type_path;
pub use type_path::{DynamicTypePath, TypePath, TypePathTable};

mod type_struct;
pub use type_struct::Type;

mod attributes;
pub use attributes::CustomAttributes;

mod generics;
pub use generics::{ConstParamInfo, GenericInfo, Generics, TypeParamInfo};

mod fields;
pub use fields::{FieldId, NamedField, UnnamedField};

mod opaque_info;
pub use opaque_info::OpaqueInfo;

mod struct_info;
pub use struct_info::StructInfo;

mod tuple_struct_info;
pub use tuple_struct_info::TupleStructInfo;

mod tuple_info;
pub use tuple_info::TupleInfo;

mod list_info;
pub use list_info::ListInfo;

mod array_info;
pub use array_info::ArrayInfo;

mod map_info;
pub use map_info::MapInfo;

mod set_info;
pub use set_info::SetInfo;

mod variant_info;
pub use variant_info::{
    StructVariantInfo, TupleVariantInfo, UnitVariantInfo, VariantInfo, VariantKind,
    VariantKindError,
};

mod enum_info;
pub use enum_info::EnumInfo;

mod type_info_impl;
pub use type_info_impl::{ReflectKind, ReflectKindError, TypeInfo};

mod typed;
pub use typed::{DynamicTyped, MaybeTyped, Typed};

// mod type_info_stack;
// pub(crate) use type_info_stack::TypeInfoStack;
