use crate::{
    Reflect,
    access::{AccessError, AccessPath, OffsetAccessor, ParseError},
    ops::{Array, Enum, List, Struct, Tuple, TupleStruct},
};
use alloc::vec::Vec;
use core::fmt;

/// An error returned from a failed path access.
#[derive(Debug, PartialEq, Eq)]
pub enum PathAccessError<'a> {
    /// An error caused by an invalid path string that couldn't be parsed.
    /// see [`ParseError`] for details.
    ParseError(ParseError<'a>),
    /// An error caused by trying to access a path that's not able to be accessed,
    /// see [`AccessError`] for details.
    AccessError(AccessError<'a>),
    /// An error that occurs when a type cannot downcast to a given type.
    InvalidDowncast,
}

impl fmt::Display for PathAccessError<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseError(err) => fmt::Display::fmt(err, f),
            Self::AccessError(err) => fmt::Display::fmt(err, f),
            Self::InvalidDowncast => {
                f.write_str("Can't downcast result of access to the given type")
            }
        }
    }
}

impl core::error::Error for PathAccessError<'_> {}

impl<'a> From<ParseError<'a>> for PathAccessError<'a> {
    #[inline]
    fn from(value: ParseError<'a>) -> Self {
        Self::ParseError(value)
    }
}

impl<'a> From<AccessError<'a>> for PathAccessError<'a> {
    #[inline]
    fn from(value: AccessError<'a>) -> Self {
        Self::AccessError(value)
    }
}

/// Reusable path accessor, wrapper of [`Vec<OffsetAccessor>`] .
///
/// [`OffsetAccessor`] and [`Accessor`] only allow access to a single level,
/// while this type allows for complete path queries.
///
/// Unlike [`ReflectPathAccess`], this container parses the path string only once during initialization.
/// However, for non-static strings, it requires copying for storage.
///
/// [`ReflectPathAccess`]: crate::access::ReflectPathAccess
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PathAccessor(Vec<OffsetAccessor<'static>>);

impl From<Vec<OffsetAccessor<'static>>> for PathAccessor {
    #[inline]
    fn from(value: Vec<OffsetAccessor<'static>>) -> Self {
        Self(value)
    }
}

impl PathAccessor {
    /// Parse the path string and create an [`PathAccessor`].
    /// If the path is incorrect, it will return [`ParseError`].
    ///
    /// This function will create a [`String`] for each path segment.
    /// For '&'static str' or `impl AccessPath<'static>`,
    /// consider using ['parse_static'] for better performance.
    ///
    /// [`Vec::shrink_to_fit`] will be called internally.
    ///
    /// [`String`]: alloc::string::String
    /// ['parse_static']: crate::access::PathAccessor::parse_static
    pub fn parse<'a>(path: impl AccessPath<'a>) -> Result<Self, ParseError<'a>> {
        let mut vc: Vec<OffsetAccessor> = Vec::with_capacity(10);
        for res in path.parse_to_accessor() {
            vc.push(res?.into_owned());
        }
        vc.shrink_to_fit();
        Ok(Self(vc))
    }

    /// Parse the path and create an [`PathAccessor`].
    /// If the path is incorrect, it will return [`ParseError`].
    ///
    /// Can only be used for '&'static str' or `impl AccessPath<'static>`,
    /// internal storage string references, no need to create additional [`String`].
    ///
    /// [`Vec::shrink_to_fit`] will be called internally.
    ///
    /// [`String`]: alloc::string::String
    pub fn parse_static(path: impl AccessPath<'static>) -> Result<Self, ParseError<'static>> {
        let mut vc: Vec<OffsetAccessor> = Vec::with_capacity(10);
        for res in path.parse_to_accessor() {
            vc.push(res?);
        }
        vc.shrink_to_fit();
        Ok(Self(vc))
    }

    /// Return the length of the internal [`Vec`],
    /// which is the number of [`OffsetAccessor`].
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access<'r>(
        &self,
        base: &'r dyn Reflect,
    ) -> Result<&'r dyn Reflect, PathAccessError<'static>> {
        let mut it = base;
        for accessor in &self.0 {
            it = match accessor.access(it) {
                Ok(val) => val,
                Err(err) => return Err(PathAccessError::AccessError(err)),
            };
        }
        Ok(it)
    }

    /// Returns a mutable reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access_mut<'r>(
        &self,
        base: &'r mut dyn Reflect,
    ) -> Result<&'r mut dyn Reflect, PathAccessError<'static>> {
        let mut it = base;
        for accessor in &self.0 {
            it = match accessor.access_mut(it) {
                Ok(val) => val,
                Err(err) => return Err(PathAccessError::AccessError(err)),
            };
        }
        Ok(it)
    }

    /// Returns a typed reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access_as<'r, T: Reflect>(
        &self,
        base: &'r dyn Reflect,
    ) -> Result<&'r T, PathAccessError<'static>> {
        let res = self.access(base)?;
        match res.downcast_ref::<T>() {
            Some(val) => Ok(val),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }

    /// Returns a mutable typed reference to the value specified by `path`.
    ///
    /// The accessor itself will not change and can be reused.
    pub fn access_mut_as<'r, T: Reflect>(
        &self,
        base: &'r mut dyn Reflect,
    ) -> Result<&'r mut T, PathAccessError<'static>> {
        let res = self.access_mut(base)?;
        match res.downcast_mut::<T>() {
            Some(val) => Ok(val),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }
}

impl fmt::Display for PathAccessor {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for it in &self.0 {
            fmt::Display::fmt(&it.accessor, f)?;
        }
        Ok(())
    }
}

pub trait ReflectPathAccess {
    /// Returns a reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access<'a, 'b>(
        &'a self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a dyn Reflect, PathAccessError<'b>>;

    /// Returns a mutable reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access_mut<'a, 'b>(
        &'a mut self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a mut dyn Reflect, PathAccessError<'b>>;

    /// Returns a typed reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access_as<'a, 'b, T: Reflect>(
        &'a self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a T, PathAccessError<'b>>;

    /// Returns a mutable typed reference to the value specified by `path`.
    ///
    /// See [`ReflectPathAccess`]
    fn access_mut_as<'a, 'b, T: Reflect>(
        &'a mut self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a mut T, PathAccessError<'b>>;
}

impl ReflectPathAccess for dyn Reflect {
    #[inline(never)]
    fn access<'a, 'b>(
        &'a self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a dyn Reflect, PathAccessError<'b>> {
        let mut it: &dyn Reflect = self;
        for res in path.parse_to_accessor() {
            let accessor = res?;
            it = accessor.access(it)?;
        }
        Ok(it)
    }

    #[inline(never)]
    fn access_mut<'a, 'b>(
        &'a mut self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a mut dyn Reflect, PathAccessError<'b>> {
        let mut it: &mut dyn Reflect = self;
        for res in path.parse_to_accessor() {
            let accessor = res?;
            it = accessor.access_mut(it)?;
        }
        Ok(it)
    }

    #[inline(never)]
    fn access_as<'a, 'b, T: Reflect>(
        &'a self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a T, PathAccessError<'b>> {
        // Not Inline `access`: Reduce compilation time.
        // Now `access` is compiled only once per impl, independent of T.
        let it = ReflectPathAccess::access(self, path)?;
        match it.downcast_ref::<T>() {
            Some(it) => Ok(it),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }

    #[inline(never)]
    fn access_mut_as<'a, 'b, T: Reflect>(
        &'a mut self,
        path: impl AccessPath<'b>,
    ) -> Result<&'a mut T, PathAccessError<'b>> {
        // Not Inline `access`: Reduce compilation time.
        // Now `access` is compiled only once per impl, independent of T.
        let it = ReflectPathAccess::access_mut(self, path)?;
        match it.downcast_mut::<T>() {
            Some(it) => Ok(it),
            None => Err(PathAccessError::InvalidDowncast),
        }
    }
}

macro_rules! impl_reflect_path_access {
    () => {
        #[inline(always)]
        fn access<'a, 'b>(
            &'a self,
            path: impl AccessPath<'b>,
        ) -> Result<&'a dyn Reflect, PathAccessError<'b>> {
            // Significantly reduce compilation time
            <dyn Reflect as ReflectPathAccess>::access(self, path)
        }

        #[inline(always)]
        fn access_mut<'a, 'b>(
            &'a mut self,
            path: impl AccessPath<'b>,
        ) -> Result<&'a mut dyn Reflect, PathAccessError<'b>> {
            // Significantly reduce compilation time
            <dyn Reflect as ReflectPathAccess>::access_mut(self, path)
        }

        #[inline(always)]
        fn access_as<'a, 'b, T: Reflect>(
            &'a self,
            path: impl AccessPath<'b>,
        ) -> Result<&'a T, PathAccessError<'b>> {
            // Significantly reduce compilation time
            <dyn Reflect as ReflectPathAccess>::access_as::<T>(self, path)
        }

        #[inline(always)]
        fn access_mut_as<'a, 'b, T: Reflect>(
            &'a mut self,
            path: impl AccessPath<'b>,
        ) -> Result<&'a mut T, PathAccessError<'b>> {
            // Significantly reduce compilation time
            <dyn Reflect as ReflectPathAccess>::access_mut_as::<T>(self, path)
        }
    };
    (dyn $name:ident) => {
        impl ReflectPathAccess for dyn $name {
            impl_reflect_path_access!();
        }
    };
    (T: $name:ident) => {
        impl<P: Sized + $name> ReflectPathAccess for P {
            impl_reflect_path_access!();
        }
    };
}

impl_reflect_path_access!(T: Reflect);

impl_reflect_path_access!(dyn Struct);
impl_reflect_path_access!(dyn TupleStruct);
impl_reflect_path_access!(dyn Tuple);
impl_reflect_path_access!(dyn List);
impl_reflect_path_access!(dyn Array);
impl_reflect_path_access!(dyn Enum);
