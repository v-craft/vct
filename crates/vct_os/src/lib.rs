#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

pub mod cfg {
    pub use vct_cfg::std;
    pub(crate) use vct_cfg::switch;

    vct_cfg::define_alias! {
        #[cfg(all(target_arch = "wasm32", feature = "web"))] => web
    }
}

cfg::std! {
    extern crate std;
}

pub mod sync;
pub mod thread;
pub mod time;

/// 重导出 web 相关库
#[doc(hidden)]
pub mod exports {
    crate::cfg::web! {
        pub use js_sys;
        pub use wasm_bindgen;
        pub use wasm_bindgen_futures;
    }
}
