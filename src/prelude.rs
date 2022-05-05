pub fn rand_f32(min: f32, max: f32) -> f32 {
    rand::random::<f32>() * (max - min) + min
}
#[cfg(test)]
mod test {
    use super::*;
    #[test]
    pub fn test_rand() {
        for i in 0..10_000 {
            let r = rand_f32(0.0, 1.0);
            assert!(r >= 0.0);
            assert!(r <= 1.0);
        }
    }
}
