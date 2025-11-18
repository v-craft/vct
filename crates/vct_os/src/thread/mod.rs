
pub use implementation::{
    sleep
};

crate::cfg::switch! {
    crate::cfg::std => {
        use std::thread as implementation;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}
