use super::{FastOctTree, Leafable};
use cgmath::{Point3, Vector3};
impl<T: Leafable> FastOctTree<T> {
    pub(crate) fn combine(mut self, other: &Self, offset: Vector3<i32>) -> Self {
        for x in 0..other.world_size() {
            for y in 0..other.world_size() {
                for z in 0..other.world_size() {
                    let get_position = Point3::new(x, y, z);
                    let set_position = Point3::new(x as i32, y as i32, z as i32) + offset;
                    if set_position.x >= 0 && set_position.y >= 0 && set_position.z >= 0 {
                        if let Some(value) = other.get(get_position) {
                            self.set(value, set_position.map(|v| v as u32));
                        }
                    }
                }
            }
        }
        self
    }
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn two_empty() {
        let e0 = FastOctTree::<u32>::new();
        let e1 = FastOctTree::<u32>::new();
        let combined = e0.combine(&e1, Vector3::new(0, 0, 0));
        assert_eq!(combined.world_size(), 0);
    }
    #[test]
    fn first_empty_second_full() {
        let e0 = FastOctTree::<u32>::new();
        let mut t1 = FastOctTree::<u32>::new();
        t1.set(1, Point3::new(0, 0, 0));
        let combined = e0.combine(&t1, Vector3::new(0, 0, 0));
        assert_eq!(combined.world_size(), 1);
        assert_eq!(combined.get(Point3::new(0, 0, 0)), Some(1));
    }
    #[test]
    fn offset() {
        let e0 = FastOctTree::<u32>::new();
        let mut t1 = FastOctTree::<u32>::new();
        t1.set(10, Point3::new(0, 0, 0));
        let offset = Vector3::new(0, 0, 10);

        let combined = e0.combine(&t1, offset);
        assert_eq!(combined.world_size(), 16);
        for x in 0..16u32 {
            for y in 0..16 {
                for z in 0..16 {
                    let get = Point3::new(x, y, z);
                    if get.x == offset.x as u32
                        && get.y == offset.y as u32
                        && get.z == offset.z as u32
                    {
                        assert_eq!(combined.get(get), Some(10))
                    } else {
                        assert_eq!(combined.get(get), None);
                    }
                }
            }
        }
    }
}
