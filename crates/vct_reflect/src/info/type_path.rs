use core::fmt;

/// 一个类型名的静态访问器
/// 
/// 通常由宏自动生成实现，但也可以手动实现
/// 
/// TODO: 宏
pub trait TypePath: 'static {
    /// 返回完整的类型名
    /// 
    /// 例：`Option<Vec<usize>>` -> `"core::option::Option<alloc::vec::Vec<usize>>"`
    fn type_path() -> &'static str;

    /// 返回不带模块路径的短类型名
    /// 
    /// 例：`Option<Vec<usize>>` -> `"Option<Vec<usize>>"`
    fn short_type_path() -> &'static str;

    /// 返回类型标识，匿名类型返回 [`None`]
    /// 
    /// 例：`Option<Vec<usize>>` -> `"Option"`
    fn type_ident() -> Option<&'static str> {
        None
    }

    /// 返回类型所在 crate 的名称，匿名类型返回 [`None`]
    /// 
    /// 例：`Option<Vec<usize>>` -> `"core"`
    fn crate_name() -> Option<&'static str> {
        None
    }

    /// 返回类型所在的模块路径
    /// 
    /// 例：`Option<Vec<usize>>` -> `"core::option"`
    fn module_path() -> Option<&'static str> {
        None
    }
}

/// 用于动态分发的 [`TypePath`]
/// 
/// [`TypePath`] 中的函数不含 `&self`，因此它无法生成特征对象。
/// 此类型用于解决此问题，下列函数直接调用 `TypePath` 的实现。 
pub trait DynamicTypePath {
    /// See [`TypePath::type_path`].
    fn reflect_type_path(&self) -> &str;

    /// See [`TypePath::short_type_path`].
    fn reflect_short_type_path(&self) -> &str;

    /// See [`TypePath::type_ident`].
    fn reflect_type_ident(&self) -> Option<&str>;

    /// See [`TypePath::crate_name`].
    fn reflect_crate_name(&self) -> Option<&str>;

    /// See [`TypePath::module_path`].
    fn reflect_module_path(&self) -> Option<&str>;
}

impl<T: TypePath> DynamicTypePath for T {
    #[inline]
    fn reflect_type_path(&self) -> &str {
        Self::type_path()
    }

    #[inline]
    fn reflect_short_type_path(&self) -> &str {
        Self::short_type_path()
    }

    #[inline]
    fn reflect_type_ident(&self) -> Option<&str> {
        Self::type_ident()
    }

    #[inline]
    fn reflect_crate_name(&self) -> Option<&str> {
        Self::crate_name()
    }

    #[inline]
    fn reflect_module_path(&self) -> Option<&str> {
        Self::module_path()
    }
}

/// 提供 [`TypePath`] 方法的直接访问
#[derive(Clone, Copy)]
pub struct TypePathTable {
    // 假设 type_path 会被频繁访问，直接缓存结果
    type_path: &'static str,
    short_type_path: fn() -> &'static str,
    type_ident: fn() -> Option<&'static str>,
    crate_name: fn() -> Option<&'static str>,
    module_path: fn() -> Option<&'static str>,
}

impl TypePathTable {
    /// 指定类型并创建新 Table。
    pub fn of<T: TypePath + ?Sized>() -> Self {
        Self {
            type_path: T::type_path(),
            short_type_path: T::short_type_path,
            type_ident: T::type_ident,
            crate_name: T::crate_name,
            module_path: T::module_path,
        }
    }

    /// 参考 [`TypePath::type_path`]
    #[inline(always)]
    pub fn path(&self) -> &'static str {
        self.type_path
    }

    /// 参考 [`TypePath::short_type_path`]
    #[inline]
    pub fn short_path(&self) -> &'static str {
        (self.short_type_path)()
    }

    /// 参考 [`TypePath::type_ident`]
    #[inline]
    pub fn ident(&self) -> Option<&'static str> {
        (self.type_ident)()
    }

    /// 参考 [`TypePath::crate_name`]
    #[inline]
    pub fn crate_name(&self) -> Option<&'static str> {
        (self.crate_name)()
    }

    /// 参考 [`TypePath::module_path`]
    #[inline]
    pub fn module_path(&self) -> Option<&'static str> {
        (self.module_path)()
    }
}

impl fmt::Debug for TypePathTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypePathVtable")
            .field("type_path", &self.type_path)
            .field("short_type_path", &(self.short_type_path)())
            .field("type_ident", &(self.type_ident)())
            .field("crate_name", &(self.crate_name)())
            .field("module_path", &(self.module_path)())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn size_of_type_path_table() {
        let size = size_of::<TypePathTable>();
        // 16 byte alignment
        assert_eq!(size, 48usize, "Expected size_of::<TypePathTable>() is 48, instead of {size}.");
    }

}
