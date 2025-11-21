use crate::DynamicTypePath;

pub trait PartialReflect: DynamicTypePath + Send + Sync + 'static {}
