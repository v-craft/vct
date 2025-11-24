#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![no_std]

/// Used to represent a disabled conditional compilation block
///
/// # Example
///
/// ```
/// use vct_cfg as cfg;
/// 
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

/// Used to represent a enabled condition compilation block
///
/// # Example
///
/// ```
/// use vct_cfg as cfg;
/// 
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

/// A conditional compilation macro similar to `switch``
///
/// # Example
///
/// ```
/// use vct_cfg as cfg;
/// 
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
/// assert!(x == 1 || x == 10);
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

/// Define aliases for compilation options
///
/// # Example
///
/// ```
/// use vct_cfg as cfg;
/// 
/// cfg::define_alias!{
///     #[cfg(test)] => enable_test,
/// };
///
/// // `enable_test` is eq to 'cfg::enabled' in testing.
/// // Otherwise it is eq to 'cfg::disabled'.
/// let mut x = false;
/// enable_test!{ x = true; };
/// 
/// // Docs test is not Unit Test.
/// // So `enable_test!` is eq to 'cfg::disabled'.
/// assert!(x == false);
/// ```
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
