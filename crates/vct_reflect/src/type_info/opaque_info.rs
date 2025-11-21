use crate::{
     Reflect,
    type_info::{
        Type, TypePath, Generics,
        docs_macro::impl_docs_fn,
        generics::impl_generic_fn,
        type_struct::impl_type_fn,
    },
};

/// 存储不透明类型（无法再细分）的信息
#[derive(Debug, Clone)]
pub struct OpaqueInfo {
    ty: Type,
    generics: Generics,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}


impl OpaqueInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    // 根据类型创建新对象，泛型信息初始化为空
    #[inline]
    pub fn new<T: Reflect + TypePath + ?Sized>() -> Self {
        Self {
            ty: Type::of::<T>(),
            generics: Generics::new(),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }
}

