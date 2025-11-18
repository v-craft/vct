
pub use implementation::Instant;

crate::cfg::switch! {
    crate::cfg::web => {
        use web_time as implementation;
    }
    crate::cfg::std => {
        use std::time as implementation;
    }
    _ => {
        compile_error!("This platform is not supported");
    }
}


