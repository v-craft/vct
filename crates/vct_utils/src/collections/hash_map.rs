//! 基于 [hashbrown] 的实现，提供新的 [`HashMap`]。
//! 此 [`HashMap`] 默认使用 [`FixedHash`] 而不是 [`RandomState`]。

use core::{
    fmt::Debug,
    hash::{BuildHasher, Hash},
    ops::{Deref, DerefMut, Index},
};

// 固定的哈希生成器
use crate::hash::FixedHash;

pub use crate::hash::{DefaultHasher, RandomState};

use hashbrown::{Equivalent, hash_map as hb};

// 重导出 API
pub use hb::{
    Drain, EntryRef, ExtractIf, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, OccupiedEntry,
    OccupiedError, RawEntryBuilder, RawEntryBuilderMut, RawEntryMut, RawOccupiedEntryMut,
    VacantEntry, Values, ValuesMut,
};

/// 一个默认使用 `FixedHash` 的简化别名
pub type Entry<'a, K, V, S = FixedHash> = hb::Entry<'a, K, V, S>;

/// 一个基于 [`hb::HashMap`] 的 new-type，默认使用 [`FixedHash`] 作为哈希构造器
///
/// 大部分方法都直接调用底层 `hb::HashMap` 的操作，额外添加少量方法以简化操作
#[repr(transparent)]
pub struct HashMap<K, V, S = FixedHash>(hb::HashMap<K, V, S>);

impl<K, V, const N: usize> From<[(K, V); N]> for HashMap<K, V, FixedHash>
where
    K: Eq + Hash,
{
    fn from(arr: [(K, V); N]) -> Self {
        arr.into_iter().collect()
    }
}

impl<K, V> HashMap<K, V, FixedHash> {
    /// 创建一个空的 [`HashMap`] 。
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    ///
    /// let map = HashMap::new();
    /// # // 文档测试
    /// # let mut map = map;
    /// # map.insert(0usize, "foo");
    /// # assert_eq!(map.get(&0), Some("foo").as_ref());
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self::with_hasher(FixedHash)
    }

    /// 创建一个空的 [`HashMap`] 并预留指定的容量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    ///
    /// let map = HashMap::with_capacity(5);
    /// # // 文档测试
    /// # let mut map = map;
    /// # map.insert(0usize, "foo");
    /// # assert_eq!(map.get(&0), Some("foo").as_ref());
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_capacity_and_hasher(capacity, FixedHash)
    }
}

// --------------------------------------------------
// ↓ 复写一遍底层方法

impl<K, V, S> Clone for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Clone,
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

impl<K, V, S> Debug for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Debug,
{
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        <hb::HashMap<K, V, S> as Debug>::fmt(&self.0, f)
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Default,
{
    #[inline]
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<K, V, S> PartialEq for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<K, V, S> Eq for HashMap<K, V, S> where hb::HashMap<K, V, S>: Eq {}

impl<K, V, S, T> FromIterator<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: FromIterator<T>,
{
    #[inline]
    fn from_iter<U: IntoIterator<Item = T>>(iter: U) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}

impl<K, V, S, T> Index<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Index<T>,
{
    type Output = <hb::HashMap<K, V, S> as Index<T>>::Output;

    #[inline]
    fn index(&self, index: T) -> &Self::Output {
        self.0.index(index)
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: IntoIterator,
{
    type Item = <hb::HashMap<K, V, S> as IntoIterator>::Item;
    type IntoIter = <hb::HashMap<K, V, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S>
where
    &'a hb::HashMap<K, V, S>: IntoIterator,
{
    type Item = <&'a hb::HashMap<K, V, S> as IntoIterator>::Item;
    type IntoIter = <&'a hb::HashMap<K, V, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.0).into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S>
where
    &'a mut hb::HashMap<K, V, S>: IntoIterator,
{
    type Item = <&'a mut hb::HashMap<K, V, S> as IntoIterator>::Item;
    type IntoIter = <&'a mut hb::HashMap<K, V, S> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.0).into_iter()
    }
}

impl<K, V, S, T> Extend<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: Extend<T>,
{
    #[inline]
    fn extend<U: IntoIterator<Item = T>>(&mut self, iter: U) {
        self.0.extend(iter);
    }
}

impl<K, V, S> From<hb::HashMap<K, V, S>> for HashMap<K, V, S> {
    #[inline]
    fn from(value: hb::HashMap<K, V, S>) -> Self {
        Self(value)
    }
}

impl<K, V, S> From<HashMap<K, V, S>> for hb::HashMap<K, V, S> {
    #[inline]
    fn from(value: HashMap<K, V, S>) -> Self {
        value.0
    }
}

impl<K, V, S> Deref for HashMap<K, V, S> {
    type Target = hb::HashMap<K, V, S>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<K, V, S> DerefMut for HashMap<K, V, S> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "serialize")]
impl<K, V, S> serde::Serialize for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: serde::Serialize,
{
    #[inline]
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
    where
        T: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serialize")]
impl<'de, K, V, S> serde::Deserialize<'de> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: serde::Deserialize<'de>,
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
impl<K, V, S, T> FromParallelIterator<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: FromParallelIterator<T>,
    T: Send,
{
    fn from_par_iter<P>(par_iter: P) -> Self
    where
        P: IntoParallelIterator<Item = T>,
    {
        Self(<hb::HashMap<K, V, S> as FromParallelIterator<T>>::from_par_iter(par_iter))
    }
}

#[cfg(feature = "rayon")]
impl<K, V, S> IntoParallelIterator for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: IntoParallelIterator,
{
    type Item = <hb::HashMap<K, V, S> as IntoParallelIterator>::Item;
    type Iter = <hb::HashMap<K, V, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K: Sync, V: Sync, S> IntoParallelIterator for &'a HashMap<K, V, S>
where
    &'a hb::HashMap<K, V, S>: IntoParallelIterator,
{
    type Item = <&'a hb::HashMap<K, V, S> as IntoParallelIterator>::Item;
    type Iter = <&'a hb::HashMap<K, V, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        (&self.0).into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<'a, K: Sync, V: Sync, S> IntoParallelIterator for &'a mut HashMap<K, V, S>
where
    &'a mut hb::HashMap<K, V, S>: IntoParallelIterator,
{
    type Item = <&'a mut hb::HashMap<K, V, S> as IntoParallelIterator>::Item;
    type Iter = <&'a mut hb::HashMap<K, V, S> as IntoParallelIterator>::Iter;

    fn into_par_iter(self) -> Self::Iter {
        (&mut self.0).into_par_iter()
    }
}

#[cfg(feature = "rayon")]
impl<K, V, S, T> ParallelExtend<T> for HashMap<K, V, S>
where
    hb::HashMap<K, V, S>: ParallelExtend<T>,
    T: Send,
{
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = T>,
    {
        <hb::HashMap<K, V, S> as ParallelExtend<T>>::par_extend(&mut self.0, par_iter);
    }
}

impl<K, V, S> HashMap<K, V, S> {
    /// 创建一个空的 [`HashMap`] 并使用指定的哈希构造器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// # use vct_os::hash::FixedHash as SomeHasher;
    ///
    /// let map = HashMap::with_hasher(SomeHasher);
    /// # // doc test
    /// # let mut map = map;
    /// # map.insert(0usize, "foo");
    /// # assert_eq!(map.get(&0), Some("foo").as_ref());
    /// ```
    #[inline]
    pub const fn with_hasher(hash_builder: S) -> Self {
        Self(hb::HashMap::with_hasher(hash_builder))
    }

    /// 创建一个空的 [`HashMap`]，预留指定容量并使用指定的哈希构造器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// # use vct_os::hash::FixedHash as SomeHasher;
    ///
    /// let map = HashMap::with_capacity_and_hasher(5, SomeHasher);
    /// # // doc test
    /// # let mut map = map;
    /// # map.insert(0usize, "foo");
    /// # assert_eq!(map.get(&0), Some("foo").as_ref());
    /// ```
    #[inline]
    pub fn with_capacity_and_hasher(capacity: usize, hash_builder: S) -> Self {
        Self(hb::HashMap::with_capacity_and_hasher(
            capacity,
            hash_builder,
        ))
    }

    /// 返回内部使用的 [`BuildHasher`] 的引用
    #[inline]
    pub fn hasher(&self) -> &S {
        self.0.hasher()
    }

    /// 返回内部“容量”
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let map = HashMap::with_capacity(5);
    ///
    /// # // doc test
    /// # let map: HashMap<(), ()> = map;
    /// # assert!(map.capacity() >= 5);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.0.capacity()
    }

    /// 获取`&'a K` 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for key in map.keys() {
    ///     // foo, bar, baz （不保证顺序）
    /// }
    /// # assert_eq!(map.keys().count(), 3);
    /// ```
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        self.0.keys()
    }

    /// 返回 `&'a V` 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for key in map.values() {
    ///     // 0, 1, 2 （不保证顺序）
    /// }
    /// # assert_eq!(map.values().count(), 3);
    /// ```
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        self.0.values()
    }

    /// 返回 `&'a mut V` 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for key in map.values_mut() {
    ///     // 0, 1, 2 （不保证顺序）
    /// }
    /// # assert_eq!(map.values_mut().count(), 3);
    /// ```
    #[inline]
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        self.0.values_mut()
    }

    /// 返回 `(&'a K, &'a V)` 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for (key, value) in map.iter() {
    ///     // ("foo", 0), ("bar", 1), ("baz", 2) （不保证顺序）
    /// }
    /// # assert_eq!(map.iter().count(), 3);
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        self.0.iter()
    }

    /// 返回 `(&'a K, &'a mut V)` 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for (key, value) in map.iter_mut() {
    ///     // ("foo", 0), ("bar", 1), ("baz", 2) （不保证顺序）
    /// }
    /// # assert_eq!(map.iter_mut().count(), 3);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        self.0.iter_mut()
    }

    /// 返回容器中现有元素的数量
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// assert_eq!(map.len(), 0);
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 如果容器内没有元素，则返回 `true`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// assert!(map.is_empty());
    ///
    /// map.insert("foo", 0);
    ///
    /// assert!(!map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 清理容器，返回键值对的迭代器，保持容量不变（不释放内存）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for (key, value) in map.drain() {
    ///     // ("foo", 0), ("bar", 1), ("baz", 2) （不保证顺序）
    /// }
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn drain(&mut self) -> Drain<'_, K, V> {
        self.0.drain()
    }

    /// 保留容器中满足条件的元素，保持容量不变（不释放内存）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// map.retain(|key, value| *value == 2);
    ///
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.0.retain(f);
    }

    /// 去除满足条件的元素，返回被去除元素的迭代器,容器容量不变（不释放内存）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// let extracted = map
    ///     .extract_if(|key, value| *value == 2)
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(map.len(), 2);
    /// assert_eq!(extracted.len(), 1);
    /// ```
    #[inline]
    pub fn extract_if<F>(&mut self, f: F) -> ExtractIf<'_, K, V, F>
    where
        F: FnMut(&K, &mut V) -> bool,
    {
        self.0.extract_if(f)
    }

    /// 清理容器但保留已分配的内存
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// map.clear();
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// 消耗自身并返回 key 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for key in map.into_keys() {
    ///     // "foo", "bar", "baz" （不保证顺序）
    /// }
    /// ```
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.0.into_keys()
    }

    /// 消耗自身并返回 val 的迭代器
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// #
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// for key in map.into_values() {
    ///     // 0, 1, 2 （不保证顺序）
    /// }
    /// ```
    #[inline]
    pub fn into_values(self) -> IntoValues<K, V> {
        self.0.into_values()
    }

    /// 去除内部的 [`HashMap`](hb::HashMap)
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let map: HashMap<&'static str, usize> = HashMap::new();
    /// let map: hashbrown::HashMap<&'static str, usize, _> = map.into_inner();
    /// ```
    #[inline]
    pub fn into_inner(self) -> hb::HashMap<K, V, S> {
        self.0
    }
}

impl<K, V, S> HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    /// 将容量提升至不少于 `additional` 的量，可能会分配更多空间以避免频繁重分配
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::with_capacity(5);
    ///
    /// # let mut map: HashMap<(), ()> = map;
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
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::with_capacity(5);
    ///
    /// # let mut map: HashMap<(), ()> = map;
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
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::with_capacity(5);
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
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

    /// 获取给定 key 在 map 中的对应条目
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// let value = map.entry("foo").or_insert(0);
    /// #
    /// # assert_eq!(*value, 0);
    /// ```
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, S> {
        self.0.entry(key)
    }

    /// 获取给定键在映射中的对应条目的引用
    ///
    /// Refer to [`entry_ref`](hb::HashMap::entry_ref) for further details.
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    /// # let mut map: HashMap<&'static str, usize> = map;
    ///
    /// let value = map.entry_ref("foo").or_insert(0);
    /// #
    /// # assert_eq!(*value, 0);
    /// ```
    #[inline]
    pub fn entry_ref<'a, 'b, Q>(&'a mut self, key: &'b Q) -> EntryRef<'a, 'b, K, Q, V, S>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.entry_ref(key)
    }

    /// 返回给定 key 对应的 value 的不可变引用（如果存在）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.get("foo"), Some(&0));
    /// ```
    #[inline]
    pub fn get<Q>(&self, k: &Q) -> Option<&V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get(k)
    }

    /// 返回给定键对应的键值对的不可变引用（如果存在）
    ///
    /// Refer to [`get_key_value`](hb::HashMap::get_key_value) for further details.
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.get_key_value("foo"), Some((&"foo", &0)));
    /// ```
    #[inline]
    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(&K, &V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get_key_value(k)
    }

    /// 返回给定键对应的键值对的引用，值将是可变引用（如果存在）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.get_key_value_mut("foo"), Some((&"foo", &mut 0)));
    /// ```
    #[inline]
    pub fn get_key_value_mut<Q>(&mut self, k: &Q) -> Option<(&K, &mut V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get_key_value_mut(k)
    }

    /// 若内部存在给定键则返回 `true`
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert!(map.contains_key("foo"));
    /// ```
    #[inline]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.contains_key(k)
    }

    /// 获取给定键对应的值的可变引用（如果存在）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.get_mut("foo"), Some(&mut 0));
    /// ```
    #[inline]
    pub fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get_mut(k)
    }

    /// 一次性批量获取给定键对应的值的可变引用（如果存在）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// let result = map.get_many_mut(["foo", "bar"]);
    ///
    /// assert_eq!(result, [Some(&mut 0), Some(&mut 1)]);
    /// ```
    #[inline]
    pub fn get_many_mut<Q, const N: usize>(&mut self, ks: [&Q; N]) -> [Option<&'_ mut V>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get_many_mut(ks)
    }

    /// 一次性批量获取给定键对应的键值对的引用，值是可变引用（如果存在）
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    /// map.insert("bar", 1);
    /// map.insert("baz", 2);
    ///
    /// let result = map.get_many_key_value_mut(["foo", "bar"]);
    ///
    /// assert_eq!(result, [Some((&"foo", &mut 0)), Some((&"bar", &mut 1))]);
    /// ```
    #[inline]
    pub fn get_many_key_value_mut<Q, const N: usize>(
        &mut self,
        ks: [&Q; N],
    ) -> [Option<(&'_ K, &'_ mut V)>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.get_many_key_value_mut(ks)
    }

    /// 插入键值对
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.get("foo"), Some(&0));
    /// ```
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.0.insert(k, v)
    }

    /// 尝试插入键值对，并返回值的可变引用
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.try_insert("foo", 0).unwrap();
    ///
    /// assert!(map.try_insert("foo", 1).is_err());
    /// ```
    #[inline]
    pub fn try_insert(&mut self, key: K, value: V) -> Result<&mut V, OccupiedError<'_, K, V, S>> {
        self.0.try_insert(key, value)
    }

    /// 尝试移除键值对并返回值的拷贝，保持容器容量不变
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.remove("foo"), Some(0));
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.remove(k)
    }

    /// 移除键值对并返回键值对的拷贝，保持容量不变
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// map.insert("foo", 0);
    ///
    /// assert_eq!(map.remove_entry("foo"), Some(("foo", 0)));
    ///
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn remove_entry<Q>(&mut self, k: &Q) -> Option<(K, V)>
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        self.0.remove_entry(k)
    }

    /// 返回容器总分配的内存字节数
    ///
    /// # 例
    ///
    /// ```rust
    /// # use vct_os::collections::HashMap;
    /// let mut map = HashMap::new();
    ///
    /// assert_eq!(map.allocation_size(), 0);
    ///
    /// map.insert("foo", 0u32);
    ///
    /// assert!(map.allocation_size() >= size_of::<&'static str>() + size_of::<u32>());
    /// ```
    #[inline]
    pub fn allocation_size(&self) -> usize {
        self.0.allocation_size()
    }

    /// 插入一个键值对且不检查是否已存在
    ///
    /// # 安全性要求
    ///
    /// 键不存在时使用此方法是安全的。
    /// 键已存在时，是未定义行为。
    #[expect(
        unsafe_code,
        reason = "re-exporting unsafe method from Hashbrown requires unsafe code"
    )]
    #[inline]
    pub unsafe fn insert_unique_unchecked(&mut self, key: K, value: V) -> (&K, &mut V) {
        unsafe { self.0.insert_unique_unchecked(key, value) }
    }

    /// 尝试批量获取可变引用，但不检查它们是否都满足别名要求
    ///
    /// # 安全性要求
    ///
    /// 如果查询的 key 发生重复，行为未定义（即使未使用返回值）
    #[expect(
        unsafe_code,
        reason = "re-exporting unsafe method from Hashbrown requires unsafe code"
    )]
    #[inline]
    pub unsafe fn get_many_unchecked_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<&'_ mut V>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.0.get_many_unchecked_mut(keys) }
    }

    /// 尝试批量获取可变引用，但不检查它们是否都满足别名要求
    ///
    /// # 安全性要求
    ///
    /// 如果查询的 key 发生重复，行为未定义（即使未使用返回值）
    #[expect(
        unsafe_code,
        reason = "re-exporting unsafe method from Hashbrown requires unsafe code"
    )]
    #[inline]
    pub unsafe fn get_many_key_value_unchecked_mut<Q, const N: usize>(
        &mut self,
        keys: [&Q; N],
    ) -> [Option<(&'_ K, &'_ mut V)>; N]
    where
        Q: Hash + Equivalent<K> + ?Sized,
    {
        unsafe { self.0.get_many_key_value_unchecked_mut(keys) }
    }
}
