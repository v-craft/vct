use crate::{Reflect, info::{
    Generics, MaybeTyped, Type, TypeInfo, TypePath, docs_macro::impl_docs_fn, generics::impl_generic_fn, type_struct::impl_type_fn
}, ops::List};


/// 存储编译时列表信息的容器
#[derive(Clone, Debug)]
pub struct ListInfo {
    ty: Type,
    generics: Generics,
    item_ty: Type,
    item_info: fn() -> Option<&'static TypeInfo>,
    #[cfg(feature = "reflect_docs")]
    docs: Option<&'static str>,
}

impl ListInfo {
    impl_docs_fn!(docs);
    impl_type_fn!(ty);
    impl_generic_fn!(generics);

    /// 创建新容器
    #[inline]
    pub fn new<
        TList: TypePath + List,
        TItem: MaybeTyped + TypePath + Reflect,
    >() -> Self {
        Self {
            ty: Type::of::<TList>(),
            generics: Generics::new(),
            item_ty: Type::of::<TItem>(),
            item_info: TItem::maybe_type_info,
            #[cfg(feature = "reflect_docs")]
            docs: None,
        }
    }

    /// 获取列表项的类型信息
    #[inline]
    pub fn item_info(&self) -> Option<&'static TypeInfo> {
        (self.item_info)()
    }

    /// 获取列表自身的类型
    #[inline]
    pub fn item_ty(&self) -> Type {
        self.item_ty
    }
}
