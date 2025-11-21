use crate::{
    Reflect,
    type_data::PartialReflect,
    type_info::{
        TypePath, 
        TypeInfo,
    },
};

pub trait Typed: Reflect + TypePath {
    fn type_info() -> &'static TypeInfo;
}

pub trait MaybeTyped: PartialReflect{
    fn maybe_type_info() -> Option<&'static TypeInfo> {
        None
    }
}

// 在各类型定义处实现
// impl MaybeTyped for DynamicEnum {}
// impl MaybeTyped for DynamicTupleStruct {}
// impl MaybeTyped for DynamicStruct {}
// impl MaybeTyped for DynamicMap {}
// impl MaybeTyped for DynamicList {}
// impl MaybeTyped for DynamicArray {}
// impl MaybeTyped for DynamicTuple {}

pub trait DynamicTyped {
    /// See [`Typed::type_info`].
    fn reflect_type_info(&self) -> &'static TypeInfo;
}

impl<T: Typed> DynamicTyped for T {
    #[inline]
    fn reflect_type_info(&self) -> &'static TypeInfo {
        Self::type_info()
    }
}

