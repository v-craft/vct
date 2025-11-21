use crate::type_info::{
    Type, Generics, TypePath,
    docs_macro::impl_docs_fn,
    generics::impl_generic_fn, 
    type_struct::impl_type_fn,
};


#[derive(Clone, Debug)]
pub struct SetInfo {
    ty: Type,
    generics: Generics,
    value_ty: Type,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl SetInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    /// 创建新容器
    #[inline]
    pub fn new<
        TSet: TypePath /*+ Set*/, 
        TValue: TypePath /*+Reflect*/,
    >() -> Self {
        Self {
            ty: Type::of::<TSet>(),
            generics: Generics::new(),
            value_ty: Type::of::<TValue>(),
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取元素类型
    #[inline]
    pub fn value_ty(&self) -> Type {
        self.value_ty
    }
}
