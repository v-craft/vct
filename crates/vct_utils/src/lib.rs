#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

pub mod cfg {
    pub(crate) use vct_os::cfg::*;
    pub use vct_os::cfg::{alloc, std};
    define_alias! {
        #[cfg(feature = "parallel")] => parallel
    }
}

cfg::std! {
    extern crate std;
}

cfg::alloc! {
    extern crate alloc;
    // 容器仅在 alloc 启用时生效
    pub mod collections;
    // 额外的 map 容器也仅在 alloc 启用时生效
    mod maps;
    pub use maps::*;
}

cfg::parallel! {
    mod parallel_queue;
    pub use parallel_queue::*;
}

pub mod hash;
pub mod cell;
pub mod debug_info;
mod default;
mod once;
mod on_drop;

pub use default::default;
pub use once::OnceFlag;
pub use on_drop::OnDrop;

pub mod prelude {
    crate::cfg::alloc! {
        pub use alloc::{
            borrow::ToOwned, boxed::Box, format, string::String, string::ToString, vec, vec::Vec,
        };
    }
    // 忽略 `std::prelude` 的内容
    pub use crate::default;
    pub use crate::debug_info::DebugName;
    pub use disqualified::ShortName;
}




