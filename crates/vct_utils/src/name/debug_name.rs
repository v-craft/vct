//! 用在 debug 模式中调试类型名
//!
//! 存储与显示类型名需要启用 debug 特性
use core::fmt;
use alloc::{borrow::Cow, string::String};
use super::ShortName;

#[cfg(feature = "debug")]
use core::any::type_name;
#[cfg(not(feature = "debug"))]
const FEATURE_DISABLED: &str = "Enable the debug feature to see the name";

/// A container for storing type names.
/// 
/// - If the `debug` feature is enabled, the actual name will be used.
/// - If it is disabled, it does not store any content,
///   and any access will return a fixed FEATURE_DISABLED string.
/// 
/// Wrapper to help debugging ECS issues.
/// This is used to display the names of systems, components, ...
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DebugName {
    #[cfg(feature = "debug")]
    name: Cow<'static, str>,
}

impl DebugName {
    /// Create a new `DebugName` from a type by using its [`core::any::type_name`]
    #[inline]
    pub fn of<T>() -> Self {
        // TODO: Change to const fn if `type_name::<T>()` is stable const fn.
        DebugName {
            #[cfg(feature = "debug")]
            name: Cow::Borrowed(type_name::<T>()),
        }
    }

    /// Return the &str hold by this `DebugName`
    /// 
    /// use `ToString::to_string` to get String 
    #[inline]
    pub fn as_ref(&self) -> &str {
        #[cfg(feature = "debug")]
        return self.name.as_ref();
        #[cfg(not(feature = "debug"))]
        return FEATURE_DISABLED;
    }

    /// Get the [`ShortName`] corresponding to this debug name
    #[inline]
    pub fn short_name(&self) -> ShortName<'_> {
        #[cfg(feature = "debug")]
        return ShortName(self.name.as_ref());
        #[cfg(not(feature = "debug"))]
        return ShortName(FEATURE_DISABLED);
    }

    /// Create a new `DebugName` from a `&str`
    #[inline]
    #[cfg_attr(not(feature = "debug"), expect(unused_variables))]
    pub const fn borrowed(value: &'static str) -> Self {
        DebugName {
            #[cfg(feature = "debug")]
            name: Cow::Borrowed(value),
        }
    }

    /// Create a new `DebugName` from a `String`
    #[inline]
    #[cfg_attr(not(feature = "debug"), expect(unused_variables))]
    pub fn owned(value: String) -> Self {
        DebugName {
            #[cfg(feature = "debug")]
            name: Cow::Owned(value),
        }
    }
}

impl fmt::Display for DebugName {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "debug")]
        f.write_str(self.name.as_ref())?;
        #[cfg(not(feature = "debug"))]
        f.write_str(FEATURE_DISABLED)?;

        Ok(())
    }
}

impl From<&'static str> for DebugName {
    #[inline]
    fn from(value: &'static str) -> Self {
        Self::borrowed(value)
    }
}

impl From<String> for DebugName {
    #[inline]
    fn from(value: String) -> Self {
        Self::owned(value)
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{format, string::ToString};
    use crate::cfg;

    #[test]
    fn feature_disable_short_name() {
        let feature_disable = "Enable the debug feature to see the name";
        let s = ShortName(feature_disable);
        assert_eq!(s.to_string(), feature_disable);
    }

    #[test]
    fn debug_name() {
        // test: of borrowed owned as_ref to_string display short_name
        let k = DebugName::of::<u64>();
        let displayed = format!("{}", k);
        let s =  "12345";
        let k1 = DebugName::borrowed(s);
        let k2 = DebugName::owned(s.to_string());
        let k3 = DebugName::borrowed("my::module::TypeName");
        let short = k3.to_string();

        assert_eq!(k1.as_ref(), k2.as_ref());
        cfg::debug!{
            if {
                assert_eq!(k.as_ref(), "u64");
                assert_eq!(displayed, "u64");
                assert_eq!(k1.as_ref(), "12345");
                assert_eq!(k2.as_ref(), "12345");
                assert_eq!(k1.to_string(), "12345");
                assert_eq!(k2.to_string(), "12345");
                assert_eq!(short, "TypeName");
            } else {
                assert_eq!(k.as_ref(), FEATURE_DISABLED);
                assert_eq!(displayed, FEATURE_DISABLED);
                assert_eq!(k1.as_ref(), FEATURE_DISABLED);
                assert_eq!(k2.as_ref(), FEATURE_DISABLED);
                assert_eq!(k1.to_string(), FEATURE_DISABLED);
                assert_eq!(k2.to_string(), FEATURE_DISABLED);
                assert_eq!(short, FEATURE_DISABLED);
            }
        }

        // test: from
        let k3: DebugName = s.into();
        let k4: DebugName = s.to_string().into();
        let k5: DebugName = <Cow<'static, str>>::from(s).into();
        let k6: DebugName = <Cow<'static, str>>::from(s.to_string()).into();
        assert_eq!(k1, k3);
        assert_eq!(k2, k4);
        assert_eq!(k1, k5);
        assert_eq!(k2, k6);
    }

}
