use crate::Reflect;
use alloc::boxed::Box;
/// A trait that enables types to be dynamically constructed from reflected data.
///
/// The type that supports Reflect should also implement this Trait.
pub trait FromReflect: Reflect + Sized {
    /// Constructs a concrete instance of `Self` from a reflected value.
    fn from_reflect(reflect: &dyn Reflect) -> Option<Self>;

    /// Attempts to downcast the given value to `Self` using,
    /// constructing the value using [`from_reflect`] if that fails.
    fn take_from_reflect(reflect: Box<dyn Reflect>) -> Result<Self, Box<dyn Reflect>> {
        if reflect.is::<Self>() {
            // TODO: Use downcast_uncheck to reduce once type check
            // `Any::downcast_uncheck` is unstable now.
            Ok(*reflect.downcast::<Self>().unwrap())
        } else {
            match Self::from_reflect(reflect.as_ref()) {
                Some(success) => Ok(success),
                None => Err(reflect),
            }
        }
    }
}
