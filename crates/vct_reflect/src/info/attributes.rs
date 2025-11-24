use core::{
    any::TypeId,
    fmt,
};
use alloc::boxed::Box;
use vct_utils::collections::TypeIdMap;

use crate::Reflect;

/// 单个自定义属性
struct CustomAttribute {
    value: Box<dyn Reflect>,
}

impl CustomAttribute {
    /// 创建新对象
    #[inline]
    pub fn new<T: Reflect>(value: T) -> Self {
        Self {
            value: Box::new(value),
        }
    }

    /// 获取内部值的引用，需指定类型
    #[inline]
    pub fn value<T: Reflect>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }

    /// 获取内部值的 [`&dyn Reflect`]
    #[inline]
    pub fn reflect_value(&self) -> &dyn Reflect {
        &*self.value
    }
}

impl fmt::Debug for CustomAttribute {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.debug(f)
    }
}

/// 用于记录自定义属性的容器
#[derive(Default)]
pub struct CustomAttributes {
    attributes: TypeIdMap<CustomAttribute>,
}

impl CustomAttributes {
    /// 添加属性
    #[inline]
    pub fn with_attribute<T: Reflect>(mut self, value: T) -> Self {
        self.attributes.insert(TypeId::of::<T>(), CustomAttribute::new(value));
        self
    }

    /// 获取内部数据的迭代器
    #[inline]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&TypeId, &dyn Reflect)> {
        self.attributes
            .iter()
            .map(|(key, val)|(key, val.reflect_value()))
    }

    /// 查询是否包含某种属性
    #[inline]
    pub fn contains<T: Reflect>(&self) -> bool {
        self.attributes.contains_key(&TypeId::of::<T>())
    }

    /// 查询是否包含某种属性
    #[inline]
    pub fn contains_by_id(&self, id: TypeId) -> bool {
        self.attributes.contains_key(&id)
    }

    /// 获取某个自定义属性
    #[inline]
    pub fn get<T: Reflect>(&self) -> Option<&T> {
        self.attributes.get(&TypeId::of::<T>())?.value::<T>()
    }

    /// 获取某个自定义属性
    #[inline]
    pub fn get_by_id(&self, id: TypeId) -> Option<&dyn Reflect> {
        Some(self.attributes.get(&id)?.reflect_value())
    }

    /// 获取内部属性数量
    #[inline]
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// 查询内部属性是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

impl fmt::Debug for CustomAttributes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.attributes.values()).finish()
    }
}

macro_rules! impl_custom_attributes_fn {
    ($field:ident) => {
        $crate::info::attributes::impl_custom_attributes_fn!(self => &self.$field);
    };
    ($self:ident => $expr:expr) => {
        /// 根据自身返回属性
        #[inline]
        pub fn custom_attributes($self: &Self) -> &$crate::info::CustomAttributes {
            $expr
        }

        /// 获取属性
        pub fn get_attribute<T: $crate::Reflect>($self: &Self) -> Option<&T> {
            $self.custom_attributes().get::<T>()
        }

        /// 获取属性
        pub fn get_attribute_by_id($self: &Self, id: ::core::any::TypeId) -> Option<&dyn $crate::Reflect> {
            $self.custom_attributes().get_by_id(id)
        }

        /// 判断是否含有某个属性
        pub fn has_attribute<T: $crate::Reflect>($self: &Self) -> bool {
            $self.custom_attributes().contains::<T>()
        }

        /// 判断是否含有某个属性
        pub fn has_attribute_by_id($self: &Self, id: ::core::any::TypeId) -> bool {
            $self.custom_attributes().contains_by_id(id)
        }
    };
}

macro_rules! impl_with_custom_attributes {
    ($field:ident) => {
        /// 修改属性（覆盖，而非添加）
        #[inline]
        pub fn with_custom_attributes(self, attributes: CustomAttributes) -> Self {
            Self {
                $field: Arc::new(attributes),
                ..self
            }
        }
    };
}

pub(crate) use impl_custom_attributes_fn;
pub(crate) use impl_with_custom_attributes;

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::size_of;

    #[test]
    fn size_of_custom_attributes() {
        let map_size = size_of::<TypeIdMap<CustomAttributes>>();
        let size = size_of::<CustomAttributes>();
        assert_eq!(map_size, size);
        assert_eq!(size, 32usize, "Expected size_of::<Generics>() is 32, instead of {size}.");
    }
}

