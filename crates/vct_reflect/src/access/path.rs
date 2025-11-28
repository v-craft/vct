use alloc::borrow::Cow;
use core::fmt;

use crate::access::OffsetAccessor;

/// A Interface for representing path parsing error information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError<'a> {
    /// Position in `path`.
    pub offset: usize,
    /// The path that the error occurred in.
    pub path: &'a str,
    /// The underlying error.
    pub error: Cow<'a, str>,
}

impl fmt::Display for ParseError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Encountered an error at offset {} while parsing `{}`: {}",
            self.offset, self.path, self.error,
        )
    }
}

impl core::error::Error for ParseError<'_> {}

/// An abstraction representing a path,
/// where the type implementing this Trait can be considered as a "Path" for path access.
///
/// This library defaults to providing implementation for [`&str`]
///
/// [`&str`]: str
pub trait AccessPath<'a> {
    /// Parse the path and return the iterator of [`OffsetAccessor`]
    fn parse_to_accessor(&self)
    -> impl Iterator<Item = Result<OffsetAccessor<'a>, ParseError<'a>>>;
}
