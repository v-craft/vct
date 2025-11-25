/// impl `docs` and `with_docs` fn
macro_rules! impl_docs_fn {
    ($field:ident) => {
        /// Get docs
        #[cfg(feature = "reflect_docs")]
        #[inline]
        pub fn docs(&self) -> Option<&'static str> {
            self.$field
        }

        /// Modify docs (overwrite, not add)
        #[cfg(feature = "reflect_docs")]
        #[inline]
        pub fn with_docs(self, $field: Option<&'static str>) -> Self {
            Self { $field, ..self }
        }
    };
}

pub(crate) use impl_docs_fn;
