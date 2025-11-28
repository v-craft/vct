// Single layer path access interface.
// Fixed interface, no need to implement.
mod accessor;
pub use accessor::{AccessError, AccessErrorKind, Accessor, OffsetAccessor};

// An abstract "Path" interface that allows users to customize "Path" that can be parsed.
mod path;
pub use path::{AccessPath, ParseError};

// impl AccessPath for &str
mod string_parser;

// Provide complete path access API
mod path_access;
pub use path_access::{PathAccessError, PathAccessor, ReflectPathAccess};
