use core::{fmt, error, any::{Any, TypeId}};
use crate::info::{
    ArrayInfo, EnumInfo, ListInfo, MapInfo, OpaqueInfo, SetInfo, StructInfo, TupleInfo, TupleStructInfo, Type, TypePathTable, generics::impl_generic_fn
};

/// 一个用于表示”类型的类型“的枚举
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReflectKind {
    Struct,
    TupleStruct,
    Tuple,
    List,
    Array,
    Map,
    Set,
    Enum,
    Opaque,
}

impl fmt::Display for ReflectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReflectKind::Struct => f.pad("struct"),
            ReflectKind::TupleStruct => f.pad("tuple struct"),
            ReflectKind::Tuple => f.pad("tuple"),
            ReflectKind::List => f.pad("list"),
            ReflectKind::Array => f.pad("array"),
            ReflectKind::Map => f.pad("map"),
            ReflectKind::Set => f.pad("set"),
            ReflectKind::Enum => f.pad("enum"),
            ReflectKind::Opaque => f.pad("opaque"),
        }
    }
}

#[derive(Debug)]
pub struct ReflectKindError {
    pub expected: ReflectKind,
    pub received: ReflectKind,
}

impl fmt::Display for ReflectKindError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kind mismatch: expected {:?}, received {:?}", self.expected, self.received)
    }
}

impl error::Error for ReflectKindError {}

#[derive(Debug, Clone)]
pub enum TypeInfo {
    /// 结构体的类型信息
    Struct(StructInfo),
    /// 结构体元组的类型信息
    TupleStruct(TupleStructInfo),
    /// 元组的类型信息
    Tuple(TupleInfo),
    /// 类列表的类型信息
    List(ListInfo),
    /// 数组的类型信息
    Array(ArrayInfo),
    /// 键值对的类型信息
    Map(MapInfo),
    /// 集合的类型信息
    Set(SetInfo),
    /// 枚举的类型信息
    Enum(EnumInfo),
    /// 不透明类型（无法再细分）的类型信息
    Opaque(OpaqueInfo),
}

macro_rules! impl_cast_method {
    ($name:ident : $kind:ident => $info:ident) => {
        /// 类型转换函数
        #[inline]
        pub fn $name(&self) -> Result<&$info, ReflectKindError> {
            match self {
                Self::$kind(info) => Ok(info),
                _ => Err(ReflectKindError{
                    expected: ReflectKind::$kind,
                    received: self.kind(),
                }),
            }
        }
    };
}

impl TypeInfo {
    impl_cast_method!(as_struct: Struct => StructInfo);
    impl_cast_method!(as_tuple_struct: TupleStruct => TupleStructInfo);
    impl_cast_method!(as_tuple: Tuple => TupleInfo);
    impl_cast_method!(as_list: List => ListInfo);
    impl_cast_method!(as_array: Array => ArrayInfo);
    impl_cast_method!(as_map: Map => MapInfo);
    impl_cast_method!(as_enum: Enum => EnumInfo);
    impl_cast_method!(as_opaque: Opaque => OpaqueInfo);

    impl_generic_fn!(self => match self {
        Self::Struct(info) => info.generics(),
        Self::TupleStruct(info) => info.generics(),
        Self::Tuple(info) => info.generics(),
        Self::List(info) => info.generics(),
        Self::Array(info) => info.generics(),
        Self::Map(info) => info.generics(),
        Self::Set(info) => info.generics(),
        Self::Enum(info) => info.generics(),
        Self::Opaque(info) => info.generics(),
    });
    
    /// 获取底层类型
    #[inline]
    pub fn ty(&self) -> &Type {
        match self {
            Self::Struct(info) => info.ty(),
            Self::TupleStruct(info) => info.ty(),
            Self::Tuple(info) => info.ty(),
            Self::List(info) => info.ty(),
            Self::Array(info) => info.ty(),
            Self::Map(info) => info.ty(),
            Self::Set(info) => info.ty(),
            Self::Enum(info) => info.ty(),
            Self::Opaque(info) => info.ty(),
        }
    }
    /// 获取类型 id
    #[inline]
    pub fn type_id(&self) -> TypeId {
        self.ty().id()
    }

    /// 获取类路径表
    #[inline]
    pub fn type_path_table(&self) -> &TypePathTable {
        self.ty().type_path_table()
    }

    /// 获取完整类名
    #[inline]
    pub fn type_path(&self) -> &'static str {
        self.ty().path()
    }

    /// 比较类型是否相同
    #[inline]
    pub fn is<T: Any>(&self) -> bool {
        self.ty().is::<T>()
    }

    /// 获取类型的类型
    #[inline]
    pub fn kind(&self) -> ReflectKind {
        match self {
            Self::Struct(_) => ReflectKind::Struct,
            Self::TupleStruct(_) => ReflectKind::TupleStruct,
            Self::Tuple(_) => ReflectKind::Tuple,
            Self::List(_) => ReflectKind::List,
            Self::Array(_) => ReflectKind::Array,
            Self::Map(_) => ReflectKind::Map,
            Self::Set(_) => ReflectKind::Set,
            Self::Enum(_) => ReflectKind::Enum,
            Self::Opaque(_) => ReflectKind::Opaque,
        }
    }

    #[cfg(feature = "reflect_docs")]
    #[inline]
    pub fn docs(&self) -> Option<&str> {
        match self {
            Self::Struct(info) => info.docs(),
            Self::TupleStruct(info) => info.docs(),
            Self::Tuple(info) => info.docs(),
            Self::List(info) => info.docs(),
            Self::Array(info) => info.docs(),
            Self::Map(info) => info.docs(),
            Self::Set(info) => info.docs(),
            Self::Enum(info) => info.docs(),
            Self::Opaque(info) => info.docs(),
        }
    }
}



