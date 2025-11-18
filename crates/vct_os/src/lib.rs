#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

cfg::std!{
    extern crate std;
}

cfg::alloc!{
    extern crate alloc;
}

pub mod cfg;
pub mod time;
pub mod thread;
pub mod sync;

/// 重导出 web 相关库
#[doc(hidden)]
pub mod exports {
    crate::cfg::web! {
        pub use js_sys;
        pub use wasm_bindgen;
        pub use wasm_bindgen_futures;
    }
}
