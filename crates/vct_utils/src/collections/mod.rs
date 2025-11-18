//! 提供常用的容器，不含 `Vec` 和 `String`
//! 
//! 此库仅在启用 alloc 时生效
//! 
//! # 基于 [`hashbrown`] 库的哈希容器
//! 
//! - [`HashMap`]
//! - [`HashSet`]
//! - [`HashTable`]
//! 
//! # alloc 中的容器
//! 
//! - [`BTreeMap`]
//! - [`BTreeSet`]
//! - [`BinaryHeap`]
//! - [`LinkedList`]
//! - [`VecDeque`]

pub mod hash_map;
pub mod hash_set;
pub mod hash_table;

pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use hash_table::HashTable;
pub use hashbrown::Equivalent;

pub use alloc::collections::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_hash_execution() {
        use alloc::vec::Vec;

        let mut map_1 = <HashMap<_, _>>::default();
        let mut map_2 = <HashMap<_, _>>::default();
        for i in 1..10 {
            map_1.insert(i, i);
            map_2.insert(i, i);
        }
        assert_eq!(
            map_1.iter().collect::<Vec<_>>(),
            map_2.iter().collect::<Vec<_>>()
        );
    }
}
