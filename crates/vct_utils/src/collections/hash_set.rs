//! 基于 [hashbrown] 的实现，提供新的 [`HashSet`]。
//! 此 [`HashSet`] 默认使用 [`FixedHash`] 而不是 [`RandomState`](crate::hash::RandomState)。

use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
    ops::{
        BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Deref, DerefMut, Sub,
        SubAssign,
    },
};

// 固定的哈希生成器
use crate::hash::FixedHash;

use hashbrown::{Equivalent, hash_set as hb};

// 重导出 API
pub use hb::{
    Difference, Drain, ExtractIf, Intersection, IntoIter, Iter, OccupiedEntry, SymmetricDifference,
    Union, VacantEntry,
};

/// 一个默认使用 `FixedHash` 的简化别名
pub type Entry<'a, T, S = FixedHash> = hb::Entry<'a, T, S>;

/// 一个基于 [`hb::HashSet`] 的 new-type，默认使用 [`FixedHash`] 作为哈希构造器
///
/// 大部分方法都直接调用底层 `hb::HashSet` 的操作，额外添加少量方法以简化操作
#[repr(transparent)]
pub struct HashSet<T, S = FixedHash>(hb::HashSet<T, S>);

impl<T, const N: usize> From<[T; N]> for HashSet<T, FixedHash>
where
    T: Eq + Hash,
{
    fn from(value: [T; N]) -> Self {
        value.into_iter().collect()
    }
}

impl<T> HashSet<T, FixedHash> {
    /// 创建一个空的 [`HashSet`]
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let map = HashSet::new();
    /// #
    /// # let mut map = map;
    /// # map.insert("foo");
    /// # assert_eq!(map.get("foo"), Some("foo").as_ref());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self::with_hasher(FixedHash)
    }

    /// 创建一个空的 [`HashSet`] 并指定容量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let map = HashSet::with_capacity(5);
    /// #
    /// # let mut map = map;
    /// # map.insert("foo");
    /// # assert_eq!(map.get("foo"), Some("foo").as_ref());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FixedHash)
    }
}

// --------------------------------------------------
// ↓ 复写一遍底层方法

impl<T, S> Clone for HashSet<T, S>
where
    hb::HashSet<T, S>: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.0.clone_from(&source.0);
    }
}

impl<T, S> Debug for HashSet<T, S>
where
    hb::HashSet<T, S>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <hb::HashSet<T, S> as Debug>::fmt(&self.0, f)
    }
}

impl<T, S> Default for HashSet<T, S>
where
    hb::HashSet<T, S>: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T, S> PartialEq for HashSet<T, S>
where
    hb::HashSet<T, S>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, S> Eq for HashSet<T, S> where hb::HashSet<T, S>: Eq {}

impl<T, S, X> FromIterator<X> for HashSet<T, S>
where
    hb::HashSet<T, S>: FromIterator<X>,
{
    #[inline]
    fn from_iter<U: IntoIterator<Item = X>>(iter: U) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<T, S> IntoIterator for HashSet<T, S>
where
    hb::HashSet<T, S>: IntoIterator,
{
    type Item = <hb::HashSet<T, S> as IntoIterator>::Item;

    type IntoIter = <hb::HashSet<T, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, S> IntoIterator for &'a HashSet<T, S>
where
    &'a hb::HashSet<T, S>: IntoIterator,
{
    type Item = <&'a hb::HashSet<T, S> as IntoIterator>::Item;

    type IntoIter = <&'a hb::HashSet<T, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, T, S> IntoIterator for &'a mut HashSet<T, S>
where
    &'a mut hb::HashSet<T, S>: IntoIterator,
{
    type Item = <&'a mut hb::HashSet<T, S> as IntoIterator>::Item;

    type IntoIter = <&'a mut hb::HashSet<T, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<T, S, X> Extend<X> for HashSet<T, S>
where
    hb::HashSet<T, S>: Extend<X>,
{
    #[inline]
    fn extend<U: IntoIterator<Item = X>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

impl<T, S> From<crate::collections::HashMap<T, (), S>> for HashSet<T, S> {
    #[inline]
    fn from(value: crate::collections::HashMap<T, (), S>) -> Self {
        Self(hb::HashSet::from(hashbrown::HashMap::from(value)))
    }
}

impl<T, S> From<hb::HashSet<T, S>> for HashSet<T, S> {
    #[inline]
    fn from(value: hb::HashSet<T, S>) -> Self {
        Self(value)
    }
}

impl<T, S> From<HashSet<T, S>> for hb::HashSet<T, S> {
    #[inline]
    fn from(value: HashSet<T, S>) -> Self {
        value.0
    }
}

impl<T, S> Deref for HashSet<T, S> {
    type Target = hb::HashSet<T, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, S> DerefMut for HashSet<T, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serialize")]
impl<T, S> serde::Serialize for HashSet<T, S>
where
    hb::HashSet<T, S>: serde::Serialize,
{
    #[inline]
    fn serialize<U>(&self, serializer: U) -> Result<U::Ok, U::Error>
    where
        U: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, T, S> serde::Deserialize<'de> for HashSet<T, S>
where
    hb::HashSet<T, S>: serde::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(serde::Deserialize::deserialize(deserializer)?))
    }
}

#[cfg(feature = "rayon")]
use rayon::prelude::{FromParallelIterator, IntoParallelIterator, ParallelExtend};

#[cfg(feature = "rayon")]
impl<T, S, U> FromParallelIterator<U> for HashSet<T, S>
where
    hb::HashSet<T, S>: FromParallelIterator<U>,
    U: Send,
{
    fn from_par_iter<P>(par_iter: P) -> Self
    where
        P: IntoParallelIterator<Item = U>,
    {
        Self(<hb::HashSet<T, S> as FromParallelIterator<U>>::from_par_iter(par_iter))
    }
}

#[cfg(feature = "rayon")]
impl<T, S> IntoParallelIterator for HashSet<T, S>
where
    hb::HashSet<T, S>: IntoParallelIterator,
{
    type Item = <hb::HashSet<T, S> as IntoParallelIterator>::Item;
    type Iter = <hb::HashSet<T, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, T: Sync, S> IntoParallelIterator for &'a HashSet<T, S>
where
    &'a hb::HashSet<T, S>: IntoParallelIterator,
{
    type Item = <&'a hb::HashSet<T, S> as IntoParallelIterator>::Item;
    type Iter = <&'a hb::HashSet<T, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        (&self.0).into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<T, S, U> ParallelExtend<U> for HashSet<T, S>
where
    hb::HashSet<T, S>: ParallelExtend<U>,
    U: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = U>,
    {
        <hb::HashSet<T, S> as ParallelExtend<U>>::par_extend(&mut self.0, par_iter);
    }
}

impl<T, S> HashSet<T, S> {
    /// 返回容器的容量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let map = HashSet::with_capacity(5);
    ///
    /// # let map: HashSet<()> = map;
    /// #
    /// assert!(map.capacity() >= 5);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// 返回 `&'a T` 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    /// map.insert("bar");
    /// map.insert("baz");
    ///
    /// for value in map.iter() {
    ///     // "foo", "bar", "baz" （顺序不确定）
    /// }
    /// #
    /// # assert_eq!(map.iter().count(), 3);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T> {
        self.0.iter()
    }

    /// 获取内部元素的数量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert("foo");
    ///
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 如果内部没有元素，返回 `true`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// assert!(map.is_empty());
    ///
    /// map.insert("foo");
    ///
    /// assert!(!map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 清理容器并返回内部元素的迭代器，不改变容量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    /// map.insert("bar");
    /// map.insert("baz");
    ///
    /// for value in map.drain() {
    ///     // "foo", "bar", "baz"
    ///     // Note that the above order is not guaranteed
    /// }
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, T> {
        self.0.drain()
    }

    /// 仅保留符合条件的元素，不改变容量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    /// map.insert("bar");
    /// map.insert("baz");
    ///
    /// map.retain(|value| *value == "baz");
    ///
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.0.retain(f);
    }

    /// 移除符合条件的元素并返回迭代器，不改变容量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    /// map.insert("bar");
    /// map.insert("baz");
    ///
    /// let extracted = map
    ///     .extract_if(|value| *value == "baz")
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(extracted.len(), 1);
    /// ```
    #[inline]
    pub fn extract_if<F>(&mut self, f: F) -> ExtractIf<'_, T, F>
    where
        F: FnMut(&T) -> bool,
    {
        self.0.extract_if(f)
    }

    /// 清理容器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// #
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    /// map.insert("bar");
    /// map.insert("baz");
    ///
    /// map.clear();
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// 创建新容器并使用指定的哈希构造器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// # use bevy_platform::hash::FixedHash as SomeHasher;
    ///
    /// let map = HashSet::with_hasher(SomeHasher);
    /// #
    /// # let mut map = map;
    /// # map.insert("foo");
    /// # assert_eq!(map.get("foo"), Some("foo").as_ref());
    /// ```
    #[inline]
    pub const fn with_hasher(hasher: S) -> Self {
        Self(hb::HashSet::with_hasher(hasher))
    }

    /// 创建新容器并指定初始容量和哈希构造器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// # use bevy_platform::hash::FixedHash as SomeHasher;
    ///
    /// let map = HashSet::with_capacity_and_hasher(5, SomeHasher);
    /// #
    /// # let mut map = map;
    /// # map.insert("foo");
    /// # assert_eq!(map.get("foo"), Some("foo").as_ref());
    /// ```
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        Self(hb::HashSet::with_capacity_and_hasher(capacity, hasher))
    }

    /// 返回内部 [`BuildHasher`] 的引用
    #[inline]
    pub fn hasher(&self) -> &S {
        self.0.hasher()
    }

    /// 获取内部的 [`hb::HashSet`]
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let map: HashSet<&'static str> = HashSet::new();
    /// let map: hashbrown::HashSet<&'static str, _> = map.into_inner();
    /// ```
    #[inline]
    pub fn into_inner(self) -> hb::HashSet<T, S> {
        self.0
    }
}

impl<T, S> HashSet<T, S>
where
    T: Eq + Hash,
    S: BuildHasher,
{
    /// 将容量提升至不少于 `additional` 的量，可能会分配更多空间以避免频繁重分配
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::with_capacity(5);
    ///
    /// # let mut map: HashSet<()> = map;
    /// #
    /// assert!(map.capacity() >= 5);
    ///
    /// map.reserve(10);
    ///
    /// assert!(map.capacity() - map.len() >= 10);
    /// ```
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional);
    }

    /// 尝试将容量提升至不少于 `additional` 的量，可能会分配更多空间以避免频繁重分配
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::with_capacity(5);
    ///
    /// # let mut map: HashSet<()> = map;
    /// #
    /// assert!(map.capacity() >= 5);
    ///
    /// map.try_reserve(10).expect("Out of Memory!");
    ///
    /// assert!(map.capacity() - map.len() >= 10);
    /// ```
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), hashbrown::TryReserveError> {
        self.0.try_reserve(additional)
    }

    /// 尽可能削减容量到内部元素数，可能会预留部分空间
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::with_capacity(5);
    ///
    /// map.insert("foo");
    /// map.insert("bar");
    /// map.insert("baz");
    ///
    /// assert!(map.capacity() >= 5);
    ///
    /// map.shrink_to_fit();
    ///
    /// assert_eq!(map.capacity(), 3);
    /// ```
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    /// 削减容量到不低于目标的值
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.0.shrink_to(min_capacity);
    }

    /// 获取差集
    #[inline]
    pub fn difference<'a>(&'a self, other: &'a Self) -> Difference<'a, T, S> {
        self.0.difference(other)
    }

    /// 获取对称差集
    #[inline]
    pub fn symmetric_difference<'a>(&'a self, other: &'a Self) -> SymmetricDifference<'a, T, S> {
        self.0.symmetric_difference(other)
    }

    /// 获取交集
    #[inline]
    pub fn intersection<'a>(&'a self, other: &'a Self) -> Intersection<'a, T, S> {
        self.0.intersection(other)
    }

    /// 获取并集
    #[inline]
    pub fn union<'a>(&'a self, other: &'a Self) -> Union<'a, T, S> {
        self.0.union(other)
    }

    /// 如果指定元素存在，则返回 `true`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    ///
    /// assert!(map.contains("foo"));
    /// ```
    #[inline]
    pub fn contains<Q>(&self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.0.contains(value)
    }

    /// 获取指定元素的引用，如果存在
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    ///
    /// assert_eq!(map.get("foo"), Some(&"foo"));
    /// ```
    #[inline]
    pub fn get<Q>(&self, value: &Q) -> Option<&T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.0.get(value)
    }

    /// 获取指定元素的引用，如果没有则插入
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// assert_eq!(map.get_or_insert("foo"), &"foo");
    /// ```
    #[inline]
    pub fn get_or_insert(&mut self, value: T) -> &T {
        self.0.get_or_insert(value)
    }

    /// 获取指定元素的引用，如果没有则插入
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// assert_eq!(map.get_or_insert_with(&"foo", |_| "foo"), &"foo");
    /// ```
    #[inline]
    pub fn get_or_insert_with<Q, F>(&mut self, value: &Q, f: F) -> &T
    where
        Q: Hash + Equivalent<T> + ?Sized,
        F: FnOnce(&Q) -> T,
    {
        self.0.get_or_insert_with(value, f)
    }

    /// 获取指定元素对应的条目
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// let value = map.entry("foo").or_insert();
    /// #
    /// # assert_eq!(value, ());
    /// ```
    #[inline]
    pub fn entry(&mut self, value: T) -> Entry<'_, T, S> {
        self.0.entry(value)
    }

    /// 如果 `self` 与 `other` 没有共同的元素，则返回`true`
    #[inline]
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.0.is_disjoint(other)
    }

    /// 如果 `self` 是 `other` 的子集，返回 `true`
    #[inline]
    pub fn is_subset(&self, other: &Self) -> bool {
        self.0.is_subset(other)
    }

    /// 如果 `self` 是 `other` 的超集，返回 `true`
    #[inline]
    pub fn is_superset(&self, other: &Self) -> bool {
        self.0.is_superset(other)
    }

    /// 添加元素
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    ///
    /// assert!(map.contains("foo"));
    /// ```
    #[inline]
    pub fn insert(&mut self, value: T) -> bool {
        self.0.insert(value)
    }

    /// 添加元素并替换已有的等效值
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    ///
    /// assert_eq!(map.replace("foo"), Some("foo"));
    /// ```
    #[inline]
    pub fn replace(&mut self, value: T) -> Option<T> {
        self.0.replace(value)
    }

    /// 移除元素，元素存在则返回 `true`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    ///
    /// assert!(map.remove("foo"));
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn remove<Q>(&mut self, value: &Q) -> bool
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.0.remove(value)
    }

    /// 取出元素
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// map.insert("foo");
    ///
    /// assert_eq!(map.take("foo"), Some("foo"));
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn take<Q>(&mut self, value: &Q) -> Option<T>
    where
        Q: Hash + Equivalent<T> + ?Sized,
    {
        self.0.take(value)
    }

    /// 返回容器总分配的内存字节数
    ///
    /// # 例
    ///
    /// ```rust
    /// # use bevy_platform::collections::HashSet;
    /// let mut map = HashSet::new();
    ///
    /// assert_eq!(map.allocation_size(), 0);
    ///
    /// map.insert("foo");
    ///
    /// assert!(map.allocation_size() >= size_of::<&'static str>());
    /// ```
    #[inline]
    pub fn allocation_size(&self) -> usize {
        self.0.allocation_size()
    }

    /// 插入一个值且不检查是否已存在
    ///
    /// # 安全性要求
    ///
    /// 值不存在时使用此方法是安全的。
    /// 值已存在时，是未定义行为。
    #[expect(
        unsafe_code,
        reason = "re-exporting unsafe method from Hashbrown requires unsafe code"
    )]
    #[inline]
    pub unsafe fn insert_unique_unchecked(&mut self, value: T) -> &T {
        // SAFETY: safety contract is ensured by the caller.
        unsafe { self.0.insert_unique_unchecked(value) }
    }
}

impl<T, S> BitOr<&HashSet<T, S>> for &HashSet<T, S>
where
    for<'a> &'a hb::HashSet<T, S>: BitOr<&'a hb::HashSet<T, S>, Output = hb::HashSet<T, S>>,
{
    type Output = HashSet<T, S>;

    /// 返回并集
    #[inline]
    fn bitor(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        HashSet(self.0.bitor(&rhs.0))
    }
}

impl<T, S> BitAnd<&HashSet<T, S>> for &HashSet<T, S>
where
    for<'a> &'a hb::HashSet<T, S>: BitAnd<&'a hb::HashSet<T, S>, Output = hb::HashSet<T, S>>,
{
    type Output = HashSet<T, S>;

    /// 返回交集
    #[inline]
    fn bitand(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        HashSet(self.0.bitand(&rhs.0))
    }
}

impl<T, S> BitXor<&HashSet<T, S>> for &HashSet<T, S>
where
    for<'a> &'a hb::HashSet<T, S>: BitXor<&'a hb::HashSet<T, S>, Output = hb::HashSet<T, S>>,
{
    type Output = HashSet<T, S>;

    /// 返回对称差
    #[inline]
    fn bitxor(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        HashSet(self.0.bitxor(&rhs.0))
    }
}

impl<T, S> Sub<&HashSet<T, S>> for &HashSet<T, S>
where
    for<'a> &'a hb::HashSet<T, S>: Sub<&'a hb::HashSet<T, S>, Output = hb::HashSet<T, S>>,
{
    type Output = HashSet<T, S>;

    /// 返回差集
    #[inline]
    fn sub(self, rhs: &HashSet<T, S>) -> HashSet<T, S> {
        HashSet(self.0.sub(&rhs.0))
    }
}

impl<T, S> BitOrAssign<&HashSet<T, S>> for HashSet<T, S>
where
    hb::HashSet<T, S>: for<'a> BitOrAssign<&'a hb::HashSet<T, S>>,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: &HashSet<T, S>) {
        self.0.bitor_assign(&rhs.0);
    }
}

impl<T, S> BitAndAssign<&HashSet<T, S>> for HashSet<T, S>
where
    hb::HashSet<T, S>: for<'a> BitAndAssign<&'a hb::HashSet<T, S>>,
{
    #[inline]
    fn bitand_assign(&mut self, rhs: &HashSet<T, S>) {
        self.0.bitand_assign(&rhs.0);
    }
}

impl<T, S> BitXorAssign<&HashSet<T, S>> for HashSet<T, S>
where
    hb::HashSet<T, S>: for<'a> BitXorAssign<&'a hb::HashSet<T, S>>,
{
    #[inline]
    fn bitxor_assign(&mut self, rhs: &HashSet<T, S>) {
        self.0.bitxor_assign(&rhs.0);
    }
}

impl<T, S> SubAssign<&HashSet<T, S>> for HashSet<T, S>
where
    hb::HashSet<T, S>: for<'a> SubAssign<&'a hb::HashSet<T, S>>,
{
    #[inline]
    fn sub_assign(&mut self, rhs: &HashSet<T, S>) {
        self.0.sub_assign(&rhs.0);
    }
}
