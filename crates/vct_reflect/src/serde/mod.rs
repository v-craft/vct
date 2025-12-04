mod skip_field;
pub use skip_field::SkipSerde;

mod de;
mod ser;

pub use de::*;
pub use ser::*;

pub const NO_IDENT: &'static str = "_NoIdent";
