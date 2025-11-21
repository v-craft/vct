#![no_std]

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod is;
pub use is::*;

pub mod type_info;
pub mod type_data;

mod reflect;
pub use reflect::*;

