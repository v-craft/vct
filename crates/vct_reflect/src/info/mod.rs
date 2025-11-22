// 用于简化重复代码的宏
mod docs_macro;

// 获取类型路径名的容器
mod type_path;
pub use type_path::{
    TypePath, DynamicTypePath, TypePathTable
};

// 存储类型 ID 与类型名的容器
mod type_struct;
pub use type_struct::Type;

// 存储自定义属性（元数据）的容器
mod attributes;
pub use attributes::CustomAttributes;

// 存储泛型参数的容器
mod generics;
pub use generics::{
    Generics, GenericInfo, TypeParamInfo, ConstParamInfo
};

// 存储字段信息的容器
mod fields;
pub use fields::{
    NamedField, UnnamedField, FieldId
};

// 存储不可拆分的类型的信息（如整数、字符串）
mod opaque_info;
pub use opaque_info::OpaqueInfo;

// 存储结构体（命名字段结构体）信息的容器
mod struct_info;
pub use struct_info::StructInfo;

// 存储元组结构体信息的容器
mod tuple_struct_info;
pub use tuple_struct_info::TupleStructInfo;

// 存储元组信息的容器
mod tuple_info;
pub use tuple_info::TupleInfo;

// 存储列表信息的容器
mod list_info;
pub use list_info::ListInfo;

// 存储数组信息的容器
mod array_info;
pub use array_info::ArrayInfo;

// 存储键值对信息的容器
mod map_info;
pub use map_info::MapInfo;

// 存储集合信息的容器
mod set_info;
pub use set_info::SetInfo;

// 存储“变体”类型的容器（表示 enum 中的一种情况）
mod variant_info;
pub use variant_info::{
    VariantInfo, VariantType, VariantTypeError,
    StructVariantInfo, TupleVariantInfo, UnitVariantInfo,
};

// 存储枚举类型的容器
mod enum_info;
pub use enum_info::EnumInfo;

// TypeInfo
mod type_info_impl;
pub use type_info_impl::{
    TypeInfo, ReflectKindError, ReflectKind,
};

// 获取 TypeInfo 的 trait
mod typed;
pub use typed::{
    Typed, DynamicTyped, MaybeTyped,
};
