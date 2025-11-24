use core::{
    fmt,
    any::{
        Any,
        TypeId,
    }
};
use alloc::{
    borrow::Cow,
    boxed::Box,
    string::ToString,
};
use crate:: {
    Reflect, info::{
        DynamicTypePath, ReflectKind, TypeInfo, TypePath
    }, ops::{
        ApplyError, ReflectCloneError, ReflectMut, ReflectOwned, ReflectRef, array_debug, enum_debug, list_debug, map_debug, set_debug, struct_debug, tuple_debug, tuple_struct_debug
    }
};
pub trait PartialReflect: DynamicTypePath + Send + Sync + 'static {
    /// 获取目标类型
    fn get_target_type_info(&self) -> Option<&'static TypeInfo>;
    /// 类型转换
    fn as_partial_reflect(&self) -> &dyn PartialReflect;
    /// 类型转换
    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect;
    /// 类型转换
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect>;
    /// 类型转换
    fn try_as_reflect(&self) -> Option<&dyn Reflect>;
    /// 类型转换
    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect>;
    /// 类型转换
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>>;
    /// 数据应用
    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError>;
    /// 数据应用
    #[inline]
    fn apply(&mut self, value: &dyn PartialReflect) {
        PartialReflect::try_apply(self, value).unwrap();
    }
    fn reflect_ref(&self) -> ReflectRef<'_>;
    fn reflect_mut(&mut self) -> ReflectMut<'_>;
    fn reflect_owned(self: Box<Self>) -> ReflectOwned;
    fn reflect_kind(&self) -> ReflectKind;

    // fn reflect_kind(&self) -> ReflectKind {
    //     self.reflect_ref().kind()
    // }
    #[inline]
    fn is_dynamic(&self) -> bool {
        false
    }
    
    fn to_dynamic(&self) -> Box<dyn PartialReflect> {
        match self.reflect_ref() {
            ReflectRef::Struct(dyn_struct) => Box::new(dyn_struct.to_dynamic_struct()),
            ReflectRef::TupleStruct(dyn_tuple_struct) => {
                Box::new(dyn_tuple_struct.to_dynamic_tuple_struct())
            }
            ReflectRef::Tuple(dyn_tuple) => Box::new(dyn_tuple.to_dynamic_tuple()),
            ReflectRef::List(dyn_list) => Box::new(dyn_list.to_dynamic_list()),
            ReflectRef::Array(dyn_array) => Box::new(dyn_array.to_dynamic_array()),
            ReflectRef::Map(dyn_map) => Box::new(dyn_map.to_dynamic_map()),
            ReflectRef::Set(dyn_set) => Box::new(dyn_set.to_dynamic_set()),
            ReflectRef::Enum(dyn_enum) => Box::new(dyn_enum.to_dynamic_enum()),
            ReflectRef::Opaque(value) => value.reflect_clone().unwrap().into_partial_reflect(),
        }
    }

    fn reflect_clone(&self) -> Result<Box<dyn Reflect>, ReflectCloneError> {
        Err(ReflectCloneError::NotImplemented { 
            type_path: Cow::Owned(self.reflect_type_path().to_string())
        })
    }

    /// 用于整合 reflect_clone 和 take
    /// 
    /// 理论上 reflect_clone 产生的对象的底层类型必须与自身一致，因此 take 的类型为自身。
    /// 要求类型已知，因此无法用于 dyn Reflect 对象。
    /// 
    /// 如果类型支持 Clone 且效果与此函数相同，推荐直接使用 Clone 达到最佳性能。
    fn reflect_clone_take(&self) -> Result<Self, ReflectCloneError> 
    where 
        Self: TypePath + Sized,
    {
        // reflect_clone 返回的特征对象的底层类型**必须**与 Self 一致 
        self.reflect_clone()?
            .take()
            .map_err(|_| ReflectCloneError::FailedDowncast { 
                expected: Cow::Borrowed(<Self as TypePath>::type_path()), 
                received: Cow::Owned(self.reflect_type_path().to_string()),
            })
    }

    fn reflect_hash(&self) -> Option<u64> {
        None
    }

    fn reflect_partial_eq(&self, _other: &dyn PartialReflect) -> Option<bool> {
        None
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.reflect_ref() {
            ReflectRef::Struct(dyn_struct) => struct_debug(dyn_struct, f),
            ReflectRef::TupleStruct(dyn_tuple_struct) => tuple_struct_debug(dyn_tuple_struct, f),
            ReflectRef::Tuple(dyn_tuple) => tuple_debug(dyn_tuple, f),
            ReflectRef::List(dyn_list) => list_debug(dyn_list, f),
            ReflectRef::Array(dyn_array) => array_debug(dyn_array, f),
            ReflectRef::Map(dyn_map) => map_debug(dyn_map, f),
            ReflectRef::Set(dyn_set) => set_debug(dyn_set, f),
            ReflectRef::Enum(dyn_enum) => enum_debug(dyn_enum, f),
            ReflectRef::Opaque(_) => write!(f, "Reflect({})", self.reflect_type_path()),
        }
    }

}

impl dyn PartialReflect {
    #[inline]
    pub fn target_is<T: Reflect>(&self) -> bool {
        self.get_target_type_info()
            .is_some_and(|info| info.type_id() == TypeId::of::<T>())
    }

    pub fn try_downcast<T: Any>(
        self: Box<dyn PartialReflect>,
    ) -> Result<Box<T>, Box<dyn PartialReflect>> {
        self.try_into_reflect()?
            .downcast::<T>()
            .map_err(|err| err as Box<dyn PartialReflect>)
    }

    #[inline]
    pub fn try_take<T: Any>(self: Box<dyn PartialReflect>) -> Result<T, Box<dyn PartialReflect>> {
        self.try_downcast().map(|value| *value)
    }

    pub fn try_downcast_ref<T: Any>(&self) -> Option<&T> {
        self.try_as_reflect()?.downcast_ref()
    }

    pub fn try_downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.try_as_reflect_mut()?.downcast_mut()
    }
}

impl fmt::Debug for dyn PartialReflect {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.debug(f)
    }
}

impl TypePath for dyn PartialReflect {
    fn type_path() -> &'static str {
        "dyn vct_reflect::PartialReflect"
    }

    fn short_type_path() -> &'static str {
        "dyn PartialReflect"
    }
}
