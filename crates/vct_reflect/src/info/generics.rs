use alloc::{borrow::Cow, boxed::Box};
use core::ops::Deref;
use vct_os::sync::Arc;

use crate::{
    Reflect,
    info::{Type, TypePath, type_struct::impl_type_fn},
};

/// Container for storing generic type parameter information
#[derive(Clone, Debug)]
pub struct TypeParamInfo {
    ty: Type,
    name: Cow<'static, str>,
    default: Option<Type>,
}

impl TypeParamInfo {
    impl_type_fn!(ty);

    /// Create a new container
    #[inline]
    pub fn new<T: TypePath + ?Sized>(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ty: Type::of::<T>(),
            name: name.into(),
            default: None,
        }
    }

    /// Get generic parameter name
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// Get default type
    #[inline]
    pub fn default(&self) -> Option<&Type> {
        self.default.as_ref()
    }

    /// Modify default type
    #[inline]
    pub fn with_default<T: TypePath + ?Sized>(mut self) -> Self {
        self.default = Some(Type::of::<T>());
        self
    }
}

/// Container for storing generic const parameter information
#[derive(Clone, Debug)]
pub struct ConstParamInfo {
    ty: Type,
    name: Cow<'static, str>,
    // Use `Arc<>` to support `Clone` trait
    default: Option<Arc<dyn Reflect>>,
}

impl ConstParamInfo {
    impl_type_fn!(ty);

    /// Create a new container
    #[inline]
    pub fn new<T: TypePath + ?Sized>(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ty: Type::of::<T>(),
            name: name.into(),
            default: None,
        }
    }

    /// Get generic parameter name
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        &self.name
    }

    /// Get default const value
    #[inline]
    pub fn default(&self) -> Option<&dyn Reflect> {
        self.default.as_deref()
    }

    /// Modify default type
    pub fn with_default<T: Reflect + 'static>(mut self, default: T) -> Self {
        let arc = Arc::new(default);

        #[cfg(not(feature = "std"))]
        #[expect(
            unsafe_code,
            reason = "unsized coercion is unstable without using std Arc"
        )]
        // See: https://doc.rust-lang.org/alloc/sync/struct.Arc.html -- impl CoerceUnsized for Arc
        // TODO: Remove this after CoerceUnsized stabilization.
        let arc = unsafe { Arc::from_raw(Arc::into_raw(arc) as *const dyn Reflect) };

        self.default = Some(arc);
        self
    }
}

/// A Enum representing a single generic parameter
#[derive(Clone, Debug)]
pub enum GenericInfo {
    Type(TypeParamInfo),
    Const(ConstParamInfo),
}

impl From<TypeParamInfo> for GenericInfo {
    #[inline]
    fn from(value: TypeParamInfo) -> Self {
        Self::Type(value)
    }
}

impl From<ConstParamInfo> for GenericInfo {
    #[inline]
    fn from(value: ConstParamInfo) -> Self {
        Self::Const(value)
    }
}

impl GenericInfo {
    impl_type_fn!(self => match self {
        Self::Type(info) => info.ty(),
        Self::Const(info) => info.ty(),
    });

    /// Get param name
    #[inline]
    pub fn name(&self) -> &Cow<'static, str> {
        match self {
            Self::Type(info) => info.name(),
            Self::Const(info) => info.name(),
        }
    }

    /// Check if self is const parameter
    #[inline]
    pub fn is_const(&self) -> bool {
        match self {
            Self::Type(_) => false,
            Self::Const(_) => true,
        }
    }
}

/// Container for storing a list of generic type parameters
#[derive(Clone, Default, Debug)]
pub struct Generics(Box<[GenericInfo]>);

impl Generics {
    /// Create a new empty container
    #[inline]
    pub fn new() -> Self {
        Self(Box::new([]))
    }

    /// Get GenericInfo by parameter name
    ///
    /// Complexity: O(N)
    #[inline]
    pub fn get(&self, name: &str) -> Option<&GenericInfo> {
        self.0.iter().find(|info| info.name() == name)
    }

    /// Push back new paramter
    ///
    /// Complexity: O(N)
    #[inline]
    pub fn with(mut self, info: impl Into<GenericInfo>) -> Self {
        self.0 = IntoIterator::into_iter(self.0)
            .chain(core::iter::once(info.into()))
            .collect();
        self
    }
}

impl<T: Into<GenericInfo>> FromIterator<T> for Generics {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().map(Into::into).collect())
    }
}

impl Deref for Generics {
    type Target = [GenericInfo];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// impl `with_generics` and `generics`
macro_rules! impl_generic_fn {
    ($field:ident) => {
        $crate::info::generics::impl_generic_fn!(self => &self.$field);

        /// Replace its own generic information
        #[inline]
        pub fn with_generics(
            mut self,
            generics: $crate::info::Generics
        ) -> Self {
            self.$field = generics;
            self
        }
    };
    ($self:ident => $expr:expr) => {
        /// Get generics from self based on expressions
        #[inline]
        pub fn generics($self: &Self) -> &$crate::info::Generics {
            $expr
        }
    };
}

pub(crate) use impl_generic_fn;
