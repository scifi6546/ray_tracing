#[derive(Clone)]
pub struct Arena<T> {
    data: Vec<ArenaNode<T>>,
}
#[derive(Clone, Copy, Debug)]
pub struct ArenaIndex {
    index: usize,
    generation: u32,
}
#[derive(Clone, Debug)]
struct ArenaNode<T> {
    data: T,
    generation: u32,
}
impl<T> ArenaNode<T> {
    const BASE_GENERATION: u32 = 0;
}
impl<T> Arena<T> {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    /// inserts a value, If the arena is empty the root node is set
    pub fn insert(&mut self, data: T) -> ArenaIndex {
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
    pub fn get(&self, index: ArenaIndex) -> Option<&T> {
        if self.data.is_empty() {
            return None;
        }
        if index.index >= self.data.len() {
            return None;
        }
        let node = &self.data[index.index];
        if node.generation != index.generation {
            return None;
        } else {
            return Some(&node.data);
        }
    }
    pub fn get_root(&self) -> Option<&T> {
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
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let _ = Arena::<()>::new();
    }
    #[test]
    fn get_empty_root() {
        let mut a = Arena::<()>::new();
        assert_eq!(a.get_root(), None);
        assert_eq!(a.get_root_mut(), None)
    }
    #[test]
    fn get_full_root() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root(), Some(&0));
        assert_eq!(a.get_root_mut(), Some(&mut 0))
    }
    #[test]
    fn insert_values() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root(), Some(&0));
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
        assert_eq!(a.get_root(), Some(&1));
    }
    #[test]
    fn update_root() {
        let mut a = Arena::<u8>::new();
        a.insert(0);
        assert_eq!(a.get_root(), Some(&0));
        a.update_root(1);
        assert_eq!(a.get_root(), Some(&1));
    }
}
