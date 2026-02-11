use std::{collections::BTreeSet, mem::size_of};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ArenaIndex {
    index: usize,
    generation: u32,
}
#[derive(Clone, Debug)]
struct ArenaNode<T: Clone> {
    data: T,
    generation: u32,
}
impl<T: Clone> ArenaNode<T> {
    const BASE_GENERATION: u32 = 0;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArenaStats {
    pub data_len: usize,
    pub data_len_bytes: usize,
    pub num_deleted_elements: usize,
}
#[derive(Clone, Debug)]
pub struct Arena<T: Clone + std::fmt::Debug> {
    data: Vec<ArenaNode<T>>,
    deleted_indices: BTreeSet<usize>,
}
impl<T: Clone + std::fmt::Debug> Arena<T> {
    const BASE_GENERATION: u32 = ArenaNode::<T>::BASE_GENERATION;
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            deleted_indices: BTreeSet::new(),
        }
    }
    /// inserts a value, If the arena is empty the root node is set
    pub fn insert(&mut self, data: T) -> ArenaIndex {
        if let Some(index) = self.deleted_indices.pop_first() {
            self.data[index].data = data;
            self.data[index].generation += 1;
            ArenaIndex {
                index,
                generation: self.data[index].generation,
            }
        } else {
            let node = ArenaNode {
                data,
                generation: Self::BASE_GENERATION,
            };
            let index = self.data.len();
            self.data.push(node);
            ArenaIndex {
                index,
                generation: Self::BASE_GENERATION,
            }
        }
    }
    pub fn get(&self, index: ArenaIndex) -> Option<&T> {
        if self.key_exists(index) {
            let node = &self.data[index.index];
            Some(&node.data)
        } else {
            None
        }
    }
    pub fn key_exists(&self, index: ArenaIndex) -> bool {
        if index.index >= self.data.len() {
            return false;
        }
        if self.deleted_indices.contains(&index.index) {
            return false;
        }
        if index.generation != self.data[index.index].generation {
            return false;
        }
        return true;
    }
    /// Updates value in place at index with value
    pub fn update(&mut self, index: ArenaIndex, data: T) {
        assert!(self.key_exists(index));
        self.data[index.index].data = data;
    }

    ///returns a copy of the root
    pub fn get_root(&self) -> Option<T> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data[0].data.clone())
        }
    }
    pub fn get_root_ref(&self) -> Option<&T> {
        if self.data.is_empty() {
            None
        } else {
            Some(&self.data[0].data)
        }
    }

    pub fn update_root(&mut self, data: T) {
        if let Some(root) = self.data.first_mut() {
            let generation = root.generation + 1;
            *root = ArenaNode { data, generation };
        } else {
            self.data.push(ArenaNode {
                data,
                generation: Self::BASE_GENERATION,
            })
        }
    }
    pub fn delete(&mut self, index: ArenaIndex) {
        assert!(self.key_exists(index));
        self.deleted_indices.insert(index.index);
    }
    pub fn stats(&self) -> ArenaStats {
        ArenaStats {
            data_len: self.data.len(),
            data_len_bytes: self.data.len() * size_of::<T>(),
            num_deleted_elements: self.deleted_indices.len(),
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let _ = Arena::<()>::new();
    }
    #[test]
    fn get_empty() {
        let a = Arena::<()>::new();
        assert_eq!(a.get_root(), None);
    }
    #[test]
    fn get_one() {
        let mut a = Arena::<u32>::new();
        a.insert(0);
        assert_eq!(a.get_root(), Some(0));
    }
    #[test]
    fn get_ref_empty_root() {
        let a = Arena::<()>::new();
        assert_eq!(a.get_root_ref(), None);
    }
    #[test]
    fn get_ref_full_root() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root_ref(), Some(&0));
    }
    #[test]
    fn insert_values() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root_ref(), Some(&0));

        let keys = (1..100).map(|i| a.insert(i)).collect::<Vec<_>>();
        for (i, k) in keys.iter().enumerate() {
            let v = i as u8 + 1;
            assert_eq!(a.get(*k), Some(&v))
        }
    }

    #[test]
    fn update_empty_root() {
        let mut a = Arena::<u8>::new();
        a.update_root(1);
        assert_eq!(a.get_root_ref(), Some(&1));
    }
    #[test]
    fn update_root() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root_ref(), Some(&0));
        a.update_root(1);
        assert_eq!(a.get_root_ref(), Some(&1));
    }
    #[test]
    fn update() {
        let mut a = Arena::<u8>::new();
        let k = a.insert(0);
        assert_eq!(*a.get(k).unwrap(), 0);
        a.update(k, 20);
        assert_eq!(*a.get(k).unwrap(), 20);
    }
    #[test]
    fn delete() {
        let mut a = Arena::<u8>::new();
        let v = (0..10).map(|i| a.insert(i)).collect::<Vec<_>>();
        for i in 0..5 {
            a.delete(v[i]);
        }
        for i in 0..5 {
            assert!(!a.key_exists(v[i as usize]));
            assert!(a.get(v[i]).is_none())
        }
        for i in 5u8..10 {
            assert!(a.key_exists(v[i as usize]));
            assert_eq!(*a.get(v[i as usize]).unwrap(), i)
        }
        for i in 0..5 {
            a.insert(i);
        }
        assert_eq!(a.data.len(), 10);
        for i in 0..5 {
            assert!(a.get(v[i]).is_none())
        }
    }
    #[test]
    fn stats() {
        let mut a = Arena::<u32>::new();
        {
            assert_eq!(
                a.stats(),
                ArenaStats {
                    data_len: 0,
                    data_len_bytes: 0,
                    num_deleted_elements: 0
                }
            );
        }
        let v = [a.insert(0), a.insert(1)];
        assert_eq!(
            a.stats(),
            ArenaStats {
                data_len: v.len(),
                data_len_bytes: 4 * v.len(),
                num_deleted_elements: 0
            }
        );
        for index in v.iter() {
            a.delete(*index);
        }
        assert_eq!(
            a.stats(),
            ArenaStats {
                data_len: v.len(),
                data_len_bytes: 4 * v.len(),
                num_deleted_elements: v.len()
            }
        );
    }
}
