//! The types that support reflection should at least implement the following traits:
//! 
//! - [`TypePath`](crate::info::TypePath)
//! - [`Typed`](crate::info::Typed)
//! - [`PartialReflect`](crate::PartialReflect)
//! - [`Reflect`](crate::Reflect)
//! - [`GetTypeTraits`](crate::Reflect)
//! 
//! Then the following traits will be automatically implemented:
//! 
//! - [`DynamicTypePath`](crate::info::DynamicTypePath) (by `TypePath`'s impl )
//! - [`DynamicTyped`](crate::info::DynamicTyped) (by `Typed`'s impl)
//! - [`MaybeTyped`](crate::info::MaybeTyped) (by `Typed`'s impl)
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


/// Used to connect strings after macro expansion.
/// 
/// Not Inline: Reduce the compilation frequency of this code (due to different types)
#[inline(never)]
fn concat(arr: &[&str]) -> alloc::string::String {
    let mut len = 0usize;
    for &item in arr {
        len += item.len();
    }
    let mut res = alloc::string::String::with_capacity(len);
    for &item in arr {
        res.push_str(item);
    }
    res
}

