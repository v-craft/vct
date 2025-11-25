#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

extern crate alloc;

pub mod cfg {
    pub use vct_cfg::std;

    vct_cfg::define_alias! {
        #[cfg(feature = "parallel")] => parallel,
        #[cfg(feature = "debug")] => debug,
        #[cfg(feature = "serde")] => serde,
        #[cfg(feature = "rayon")] => rayon,
    }
}

cfg::std! {
    extern crate std;
}

cfg::parallel! {
    // `parallel` includes `std`
    mod parallel_queue;
    pub use parallel_queue::*;
}

pub mod name;
pub mod cell;
pub mod hash;
pub mod collections;

mod default;
mod on_drop;
mod is;

pub use default::default;
pub use on_drop::OnDrop;
pub use is::Is;

pub mod prelude {
    pub use alloc::{
        borrow::ToOwned, boxed::Box, format, string::String, string::ToString, vec, vec::Vec,
    };

    pub use crate::default;
}
