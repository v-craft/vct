use core::ops::Deref;
pub use disqualified::ShortName;

use crate::cfg;
cfg::alloc! {
    use alloc::{borrow::Cow, fmt, string::String};
}

#[cfg(feature = "debug")]
use core::any::type_name;
#[cfg(not(feature = "debug"))]
const FEATURE_DISABLED: &str = "Enable the debug feature to see the name";


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugName {
    #[cfg(feature = "debug")]
    name: Cow<'static, str>,
}

impl DebugName {
    #[inline]
    pub fn type_name<T>() -> Self {
        DebugName {
            #[cfg(feature = "debug")]
            name: Cow::Borrowed(type_name::<T>()),
        }
    }

    #[inline]
    #[cfg(feature = "debug")]
    pub fn as_string(&self) -> String {
        self.name.clone().into_owned()
    }

    #[inline]
    pub fn short_name(&self) -> ShortName<'_> {
        #[cfg(feature = "debug")]
        return ShortName(self.name.as_ref());
        #[cfg(not(feature = "debug"))]
        return ShortName(FEATURE_DISABLED);
    }

    #[inline]
    #[cfg_attr(not(feature = "debug"), expect(unused_variables))]
    pub const fn borrowed(value: &'static str) -> Self {
        DebugName {
            #[cfg(feature = "debug")]
            name: Cow::Borrowed(value),
        }
    }

    cfg::alloc! {
        #[inline]
        #[cfg_attr(not(feature = "debug"), expect(unused_variables))]
        pub fn owned(value: String) -> Self {
            DebugName {
                #[cfg(feature = "debug")]
                name: Cow::Owned(value),
            }
        }
    }
}

impl Deref for DebugName {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        #[cfg(feature = "debug")]
        return &self.name;
        #[cfg(not(feature = "debug"))]
        return FEATURE_DISABLED;
    }
}

impl From<&'static str> for DebugName {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self::borrowed(value)
    }
}

cfg::alloc! {
    impl fmt::Display for DebugName {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            #[cfg(feature = "debug")]
            f.write_str(self.name.as_ref())?;
            #[cfg(not(feature = "debug"))]
            f.write_str(FEATURE_DISABLED)?;

            Ok(())
        }
    }

    impl From<Cow<'static, str>> for DebugName {
        #[inline]
        #[cfg_attr(not(feature = "debug"), expect(unused_variables))]
        fn from(value: Cow<'static, str>) -> Self {
            Self {
                #[cfg(feature = "debug")]
                name: value,
            }
        }
    }

    impl From<String> for DebugName {
        #[inline]
        fn from(value: String) -> Self {
            Self::owned(value)
        }
    }

    impl From<DebugName> for Cow<'static, str> {
        #[inline]
        #[cfg_attr(not(feature = "debug"), expect(unused_variables))]
        fn from(value: DebugName) -> Self {
            #[cfg(feature = "debug")]
            return value.name;
            #[cfg(not(feature = "debug"))]
            return Cow::Borrowed(FEATURE_DISABLED);
        }
    }
}
