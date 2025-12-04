//! The types that support reflection should at least implement the following traits:
//!
//! - [`TypePath`](crate::info::TypePath)
//! - [`Typed`](crate::info::Typed)
//! - [`Reflect`](crate::Reflect)
//! - [`GetTypeTraits`](crate::Reflect)
//!
//! Then the following traits will be automatically implemented:
//!
//! - [`DynamicTypePath`](crate::info::DynamicTypePath) (by `TypePath`'s impl )
//! - [`DynamicTyped`](crate::info::DynamicTyped) (by `Typed`'s impl)
//! - [`Reflectable`](crate::Reflectable) (by all traits' impl)
//!
//! [`FromReflect`](crate::FromReflect) is optional but it is usually recommended to implement.
//!
//! For non-Opaque types, one (at most one) of the following traits should also be implemented:
//!
//! - [`Struct`](crate::ops::Struct)
//! - [`TupleStruct`](crate::ops::TupleStruct)
//! - [`Tuple`](crate::ops::Tuple)
//! - [`List`](crate::ops::List)
//! - [`Array`](crate::ops::Array)
//! - [`Set`](crate::ops::Set)
//! - [`Map`](crate::ops::Map)
//! - [`Enum`](crate::ops::Enum)
//!

mod native;

pub(crate) use crate::__macro_exports::alloc_utils::concat;
