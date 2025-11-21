use alloc::boxed::Box;
use crate:: {
    Reflect,
    type_data::{
        ApplyError,
    },
    type_info::{
        DynamicTypePath, TypeInfo,
    },
};
pub trait PartialReflect: DynamicTypePath + Send + Sync + 'static {
    /// 获取底层类型
    fn get_represented_type_info(&self) -> Option<&'static TypeInfo>;
    /// 类型转换
    fn into_partial_reflect(self: Box<Self>) -> Box<dyn PartialReflect>;
    /// 类型转换
    fn as_partial_reflect(&self) -> &dyn PartialReflect;
    /// 类型转换
    fn as_partial_reflect_mut(&mut self) -> &mut dyn PartialReflect;
    /// 类型转换
    fn try_into_reflect(self: Box<Self>) -> Result<Box<dyn Reflect>, Box<dyn PartialReflect>>;
    /// 类型转换
    fn try_as_reflect(&self) -> Option<&dyn Reflect>;
    /// 类型转换
    fn try_as_reflect_mut(&mut self) -> Option<&mut dyn Reflect>;
    /// 数据应用
    fn try_apply(&mut self, value: &dyn PartialReflect) -> Result<(), ApplyError>;
    /// 数据应用
    fn apply(&mut self, value: &dyn PartialReflect) {
        PartialReflect::try_apply(self, value).unwrap();
    }
    
    // ........
    
    fn is_dynamic(&self) -> bool {
        false
    }
}
