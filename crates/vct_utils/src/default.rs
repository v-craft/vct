use core::default::Default;

#[inline(always)]
pub fn default<T: Default>() -> T {
    Default::default()
}

