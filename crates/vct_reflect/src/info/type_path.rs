use core::fmt;

/// A static accessor to type paths and names.
pub trait TypePath: 'static {
    /// Returns the fully qualified path of the underlying type.
    ///
    /// `Option<Vec<usize>>` -> `"core::option::Option<alloc::vec::Vec<usize>>"`
    fn type_path() -> &'static str;

    /// Returns a short, pretty-print enabled path to the type.
    ///
    /// `Option<Vec<usize>>` -> `"Option<Vec<usize>>"`
    ///
    /// This function should return the `short name`, this may be different from [`core::any::type_name`].
    /// The result of [`core::any::type_name`] may be a full path name or a short name.
    fn type_name() -> &'static str;

    /// Returns the name of the type, or [`None`] if it is [anonymous].
    ///
    /// `Option<Vec<usize>>` -> `"Option"`
    fn type_ident() -> Option<&'static str> {
        None
    }

    /// Returns the name of the crate the type is in, or [`None`] if it is [anonymous].
    ///
    /// `Option<Vec<usize>>` -> `"core"`
    fn crate_name() -> Option<&'static str> {
        None
    }

    /// Returns the path to the module the type is in, or [`None`] if it is [anonymous].
    ///
    /// `Option<Vec<usize>>` -> `"core::option"`
    fn module_path() -> Option<&'static str> {
        None
    }
}

/// Dynamic dispatch for [`TypePath`].
/// 
/// This trait is automatically implemented for types that implement [`TypePath`].
pub trait DynamicTypePath {
    /// See [`TypePath::type_path`].
    fn reflect_type_path(&self) -> &str;

    /// See [`TypePath::type_name`].
    fn reflect_type_name(&self) -> &str;

    /// See [`TypePath::type_ident`].
    fn reflect_type_ident(&self) -> Option<&str>;

    /// See [`TypePath::crate_name`].
    fn reflect_crate_name(&self) -> Option<&str>;

    /// See [`TypePath::module_path`].
    fn reflect_module_path(&self) -> Option<&str>;
}

impl<T: TypePath> DynamicTypePath for T {
    #[inline]
    fn reflect_type_path(&self) -> &str {
        Self::type_path()
    }

    #[inline]
    fn reflect_type_name(&self) -> &str {
        Self::type_name()
    }

    #[inline]
    fn reflect_type_ident(&self) -> Option<&str> {
        Self::type_ident()
    }

    #[inline]
    fn reflect_crate_name(&self) -> Option<&str> {
        Self::crate_name()
    }

    #[inline]
    fn reflect_module_path(&self) -> Option<&str> {
        Self::module_path()
    }
}

/// Provides dynamic access to all methods on [`TypePath`].
#[derive(Clone, Copy)]
pub struct TypePathTable {
    // Most custom types are stored name by `Cow<'static, str>` and initialized on the first access.
    //
    // The default implementation only uses `type_path` frequently,
    // so only cache A here to reduce unnecessary overhead.
    type_path: &'static str,
    type_name: fn() -> &'static str,
    type_ident: fn() -> Option<&'static str>,
    crate_name: fn() -> Option<&'static str>,
    module_path: fn() -> Option<&'static str>,
}

impl TypePathTable {
    /// Creates a new table from a type.
    #[inline]
    pub fn of<T: TypePath + ?Sized>() -> Self {
        Self {
            type_path: T::type_path(),
            type_name: T::type_name,
            type_ident: T::type_ident,
            crate_name: T::crate_name,
            module_path: T::module_path,
        }
    }

    /// See [`TypePath::type_path`]
    #[inline(always)]
    pub fn path(&self) -> &'static str {
        self.type_path
    }

    /// See [`TypePath::type_name`]
    #[inline]
    pub fn name(&self) -> &'static str {
        (self.type_name)()
    }

    /// See [`TypePath::type_ident`]
    #[inline]
    pub fn ident(&self) -> Option<&'static str> {
        (self.type_ident)()
    }

    /// See [`TypePath::crate_name`]
    #[inline]
    pub fn crate_name(&self) -> Option<&'static str> {
        (self.crate_name)()
    }

    /// See [`TypePath::module_path`]
    #[inline]
    pub fn module_path(&self) -> Option<&'static str> {
        (self.module_path)()
    }
}

impl fmt::Debug for TypePathTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypePathVtable")
            .field("type_path", &self.type_path)
            .field("type_name", &(self.type_name)())
            .field("type_ident", &(self.type_ident)())
            .field("crate_name", &(self.crate_name)())
            .field("module_path", &(self.module_path)())
            .finish()
    }
}
