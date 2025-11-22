
macro_rules! impl_docs_fn {
    ($field:ident) => {
        /// 读取文档
        #[cfg(feature = "reflect_docs")]
        #[inline]
        pub fn docs(&self) -> Option<&'static str> {
            self.$field
        }

        /// 修改文档（覆盖，而非添加）
        #[cfg(feature = "reflect_docs")]
        #[inline]
        pub fn with_docs(self, $field: Option<&'static str>) -> Self {
            Self { $field, ..self }
        }
    };
}

pub(crate) use impl_docs_fn;
