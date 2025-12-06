use alloc::string::String;
use vct_reflect_derive::impl_reflect;

impl_reflect!{
    #[reflect(Opaque, full)]
    #[reflect(type_path = "alloc::string::String")]
    struct String;
}

