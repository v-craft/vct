//! Provide commonly used containers, excluding `Vec` and `String` (please use alloc directly).
//!
//! # based on [`hashbrown`]
//!
//! - [`HashMap`]
//! - [`HashSet`]
//! - [`HashTable`]
//!
//! # in alloc
//!
//! - [`BTreeMap`]
//! - [`BTreeSet`]
//! - [`BinaryHeap`]
//! - [`LinkedList`]
//! - [`VecDeque`]

pub mod hash_map;
pub mod hash_set;
pub mod hash_table;
mod maps;

pub use hashbrown::Equivalent;
pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use hash_table::HashTable;
pub use maps::{PreHashMap, TypeIdMap};

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
