#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

/// 此宏用于表示关闭的条件编译块
///
/// # 例
///
/// ```ignore
/// # use vct_os::cfg;
/// let mut x = 0;
/// assert!( !cfg::disabled!() );
/// cfg::disabled!(
///     if {
///         panic!();
///     } else {
///         x += 1;
///     }
/// );
/// cfg::disabled!{ x += 10; };
/// assert_eq!(x, 1);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! disabled {
    () => { false };
    (if { $($p:tt)* } else { $($n:tt)* }) => { $($n)* };
    ($($p:tt)*) => {};
}

/// 此宏用于表示开启的条件编译块
///
/// # 例
///
/// ```ignore
/// # use vct_os::cfg;
/// let mut x = 0;
/// assert!( cfg::enabled!() );
/// cfg::enabled!(
///     if {
///         x += 1;
///     } else {
///         panic!();
///     }
/// );
/// cfg::enabled!{ x += 10; };
/// assert_eq!(x, 11);
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! enabled {
    () => { true };
    (if { $($p:tt)* } else { $($n:tt)* }) => { $($p)* };
    ($($p:tt)*) => { $($p)* };
}

/// 一个类 switch 的条件编译宏
///
/// # 例
///
/// ```ignore
/// # use vct_os::cfg;
/// let mut x = 0;
/// cfg::switch! {
///     #[cfg(test)] => {
///         x += 1;
///     }
///     cfg::enabled => {
///         x += 10;
///     }
///     _ => {
///         x += 100;
///     }
/// }
/// ```
#[doc(hidden)]
#[macro_export]
macro_rules! switch {
    ({ $($tt:tt)* }) => {{
        $crate::switch! { $($tt)* }
    }};
    (_ => { $($output:tt)* }) => {
        $($output)*
    };
    (
        $cond:path => $output:tt
        $($( $rest:tt )+)?
    ) => {
        $cond! {
            if {
                $crate::switch! { _ => $output }
            } else {
                $(
                    $crate::switch! { $($rest)+ }
                )?
            }
        }
    };
    (
        #[cfg($cfg:meta)] => $output:tt
        $($( $rest:tt )+)?
    ) => {
        #[cfg($cfg)]
        $crate::switch! { _ => $output }
        $(
            #[cfg(not($cfg))]
            $crate::switch! { $($rest)+ }
        )?
    }
}

/// 用于给编译特性定义 `cfg::enabled` 类似的宏
///
/// # 例
///
/// ```ignore
/// cfg::define_alias!{
///     #[cfg(test)] => enable_test,
/// };
///
/// let mut x = false;
/// enable_test!{ x = true; };
/// ```
///
/// 这会提供一个 `enable_test!` 宏。如果 `#[cfg(test)]` 成立，
/// 则其等效于 `cfg::enabled` ，否则等效于 `cfg::disabled` 。
#[doc(hidden)]
#[macro_export]
macro_rules! define_alias {
    (
        #[cfg($meta:meta)] => $p:ident
        $(, $( $rest:tt )+)?
    ) => {
        $crate::define_alias! {
            #[cfg($meta)] => { $p }
            $( $($rest)+ )?
        }
    };
    (
        #[cfg($meta:meta)] => $p:ident,
    ) => {
        $crate::define_alias! {
            #[cfg($meta)] => { $p }
        }
    };
    (
        #[cfg($meta:meta)] => {
            $(#[$id_meta:meta])*
            $id:ident
        }
        $($( $rest:tt )+)?
    ) => {
        $crate::switch! {
            #[cfg($meta)] => {
                #[doc = concat!("This macro is eq to `cfg::enabled` because `#[cfg(", stringify!($meta), ")]` is currently active.")]
                #[doc(inline)]
                $(#[$id_meta])*
                pub use $crate::enabled as $id;
            }
            _ => {
                #[doc = concat!("This macro is eq to `cfg::disabled` because `#[cfg(", stringify!($meta), ")]` is _not_ currently active.")]
                #[doc(inline)]
                $(#[$id_meta])*
                pub use $crate::disabled as $id;
            }
        }

        $(
            $crate::define_alias! {
                $($rest)+
            }
        )?
    }
}

define_alias! {
    #[cfg(feature = "std")] => std,
    #[cfg(panic = "unwind")] => panic_unwind,
    #[cfg(panic = "abort")] => panic_abort,
}

#[cfg(test)]
mod test {
    use crate as cfg;

    #[test]
    fn cfg_disabled() {
        let mut x = 0;
        assert!(!cfg::disabled!());
        cfg::disabled!(
            if {
                panic!();
            } else {
                x += 1;
            }
        );
        cfg::disabled! { x += 10; };
        assert_eq!(x, 1);
    }

    #[test]
    fn cfg_enabled() {
        let mut x = 0;
        assert!(cfg::enabled!());
        cfg::enabled!(
            if {
                x += 1;
            } else {
                panic!();
            }
        );
        cfg::enabled! { x += 10; };
        assert_eq!(x, 11);
    }

    #[test]
    fn cfg_switch() {
        let mut x = 0;

        cfg::switch! {
            #[cfg(test)] => {
                x += 1;
            }
            cfg::enabled => {
                x += 10;
            }
            _ => {
                x += 100;
            }
        }

        assert_eq!(x, 1);
    }

    #[test]
    fn cfg_define_alias() {
        cfg::define_alias! {
            #[cfg(test)] => enable_test,
        };

        let mut x = false;
        enable_test! { x = !x; };
        assert!(x);
    }
}
