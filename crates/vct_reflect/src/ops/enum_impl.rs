use crate::PartialReflect;

#[derive(Default)]
pub struct DynamicEnum {}

pub trait Enum: PartialReflect {}
