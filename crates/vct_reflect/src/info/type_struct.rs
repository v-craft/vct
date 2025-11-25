use core::{
    any::{Any, TypeId},
    fmt::{Debug, Formatter},
    hash::Hash,
};

use crate::info::{TypePath, TypePathTable};

/// The base representation of a Rust type.
///
/// Including [`TypeId`] and [`TypePathTable`] .
#[derive(Copy, Clone)]
pub struct Type {
    type_path_table: TypePathTable,
    type_id: TypeId,
}

impl Type {
    /// Create a new [`Type`] from a type that implements [`TypePath`].
    #[inline]
    pub fn of<T: TypePath + ?Sized>() -> Self {
        Self {
            type_path_table: TypePathTable::of::<T>(),
            type_id: TypeId::of::<T>(),
        }
    }

    /// Returns the [`TypeId`] of the type.
    #[inline(always)]
    pub fn id(&self) -> TypeId {
        self.type_id
    }

    /// See [`TypePath::type_path`]
    #[inline]
    pub fn path(&self) -> &'static str {
        self.type_path_table.path()
    }

    /// See [`TypePath::short_type_path`]
    #[inline]
    pub fn short_path(&self) -> &'static str {
        self.type_path_table.short_path()
    }

    /// See [`TypePath::type_ident`]
    #[inline]
    pub fn ident(&self) -> Option<&'static str> {
        self.type_path_table.ident()
    }

    /// See [`TypePath::crate_name`]
    #[inline]
    pub fn crate_name(&self) -> Option<&'static str> {
        self.type_path_table.crate_name()
    }

    /// See [`TypePath::module_path`]
    #[inline]
    pub fn module_path(&self) -> Option<&'static str> {
        self.type_path_table.module_path()
    }

    /// See [`TypePathTable`]
    #[inline]
    pub fn type_path_table(&self) -> &TypePathTable {
        &self.type_path_table
    }

    /// Check if the given type matches this one.
    ///
    /// This only compares the [`TypeId`] of the types.
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        TypeId::of::<T>() == self.type_id
    }
}

/// This implementation purely relies on the [`TypeId`] of the type,
/// and not on the [`TypePath`].
impl PartialEq for Type {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Eq for Type {}

/// This implementation purely relies on the [`TypeId`] of the type,
/// and not on the [`TypePath`].
impl Hash for Type {
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

/// This implementation will only output the [`TypePath`] of the type.
impl Debug for Type {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.type_path_table.path())
    }
}

/// impl `ty` `type_id` `type_path` `type_path_table` `is`
macro_rules! impl_type_fn {
    ($field:ident) => {
        $crate::info::type_struct::impl_type_fn!(self => &self.$field);
    };
    ($self:ident => $expr:expr) => {
        /// Get underlying [`Type`].
        #[inline]
        pub fn ty($self: &Self) -> &$crate::info::Type {
            $expr
        }

        /// Get [`TypeId`]
        #[inline]
        pub fn type_id(&self) -> ::core::any::TypeId {
            self.ty().id()
        }

        /// Get type_path
        #[inline]
        pub fn type_path(&self) -> &'static str {
            self.ty().path()
        }

        /// Get [`TypePathTable`]
        #[inline]
        pub fn type_path_table(&self) -> &$crate::info::TypePathTable {
            &self.ty().type_path_table()
        }

        /// Check if the given type matches this one.
        ///
        /// This only compares the [`TypeId`] of the types.
        #[inline]
        pub fn is<T: ::core::any::Any>(&self) -> bool {
            self.ty().is::<T>()
        }
    };
}

pub(crate) use impl_type_fn;
