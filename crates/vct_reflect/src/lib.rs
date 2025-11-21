#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod is;
pub use is::*;

mod type_info;
pub use type_info::*;

mod type_data;
pub use type_data::{
    PartialReflect, Reflect
};

