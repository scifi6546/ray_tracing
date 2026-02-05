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
#[derive(Clone, Debug)]
pub struct Arena<T: Clone + std::fmt::Debug> {
    data: Vec<ArenaNode<T>>,
    deleted_indices: Vec<usize>,
}
impl<T: Clone + std::fmt::Debug> Arena<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            deleted_indices: Vec::new(),
        }
    }
    /// inserts a value, If the arena is empty the root node is set
    pub fn insert(&mut self, data: T) -> ArenaIndex {
        if !self.deleted_indices.is_empty() {
            let index = self.deleted_indices.pop().unwrap();
            self.data[index].data = data;
            self.data[index].generation += 1;
            ArenaIndex {
                index,
                generation: self.data[index].generation,
            }
        } else {
            let node = ArenaNode {
                data,
                generation: ArenaNode::<T>::BASE_GENERATION,
            };
            let index = self.data.len();
            self.data.push(node);
            ArenaIndex {
                index,
                generation: ArenaNode::<T>::BASE_GENERATION,
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
    /// Updates value at index with value
    pub fn update(&mut self, index: ArenaIndex, data: T) {
        if self.key_exists(index) {
            self.data[index.index].data = data;
        }
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
    pub fn get_root_mut(&mut self) -> Option<&mut T> {
        if self.data.is_empty() {
            None
        } else {
            Some(&mut self.data[0].data)
        }
    }
    pub fn update_root(&mut self, data: T) {
        if self.data.is_empty() {
            self.insert(data);
        } else {
            let generation = self.data[0].generation + 1;
            self.data[0] = ArenaNode { data, generation };
        }
    }
    pub fn delete(&mut self, index: ArenaIndex) {
        if self.key_exists(index) {
            self.deleted_indices.push(index.index)
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
        let mut a = Arena::<()>::new();
        assert_eq!(a.get_root_ref(), None);
        assert_eq!(a.get_root_mut(), None)
    }
    #[test]
    fn get_ref_full_root() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root_ref(), Some(&0));
        assert_eq!(a.get_root_mut(), Some(&mut 0))
    }
    #[test]
    fn insert_values() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root_ref(), Some(&0));
        assert_eq!(a.get_root_mut(), Some(&mut 0));
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
}
