use std::ops::{Index, IndexMut, Mul};
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector4 {
    pub data: [f32; 4],
}

impl Index<usize> for Vector4 {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index as usize]
    }
}
impl IndexMut<usize> for Vector4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
impl Vector4 {
    pub const ZERO: Self = Vector4::new(0., 0., 0., 0.);
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { data: [x, y, z, w] }
    }
    pub const fn x(&self) -> f32 {
        self.data[0]
    }
    pub const fn y(&self) -> f32 {
        self.data[1]
    }
    pub const fn z(&self) -> f32 {
        self.data[2]
    }
    pub const fn w(&self) -> f32 {
        self.data[3]
    }
}
impl Mul<Vector4> for f32 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Self::Output {
        todo!()
    }
}
impl Mul<Matrix4> for f32 {
    type Output = Matrix4;
    fn mul(self, rhs: Matrix4) -> Self::Output {
        todo!()
    }
}
impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;

    fn mul(self, rhs: Vector4) -> Self::Output {
        let mut output = Vector4::ZERO;
        for y in 0..4 {
            let mut dot = 0.;
            for x in 0..4 {
                let matrix_val = self[(y, x)];
                let vector_value = rhs[x];
                dot += matrix_val * vector_value;
            }
            output[y] = dot;
        }
        output
    }
}
impl Mul<Matrix4> for Matrix4 {
    type Output = Self;
    fn mul(self, rhs: Matrix4) -> Self::Output {
        todo!()
    }
}
impl Index<(usize, usize)> for Matrix4 {
    type Output = f32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.rows[index.0][index.1]
    }
}
impl Index<usize> for Matrix4 {
    type Output = Vector4;

    fn index(&self, index: usize) -> &Self::Output {
        &self.rows[index]
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Vector3 {
    data: [f32; 3],
}
impl Vector3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { data: [x, y, z] }
    }
    pub const fn x(&self) -> f32 {
        self.data[0]
    }
    pub const fn y(&self) -> f32 {
        self.data[1]
    }
    pub const fn z(&self) -> f32 {
        self.data[2]
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct Matrix4 {
    rows: [Vector4; 4],
}
impl Matrix4 {
    pub const fn from_rows(rows: [Vector4; 4]) -> Self {
        Self { rows }
    }
    pub const fn identity() -> Self {
        Self {
            rows: [
                Vector4::new(1., 0., 0., 0.),
                Vector4::new(0., 1., 0., 0.),
                Vector4::new(0., 0., 1., 0.),
                Vector4::new(0., 0., 0., 1.),
            ],
        }
    }
    pub const fn zero() -> Self {
        Self {
            rows: [Vector4::ZERO; 4],
        }
    }
    pub const fn translation(translation: Vector3) -> Self {
        Self {
            rows: [
                Vector4::new(1., 0., 0., translation.x()),
                Vector4::new(0., 1., 0., translation.y()),
                Vector4::new(0., 0., 1., translation.z()),
                Vector4::new(0., 0., 0., 1.),
            ],
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;

    fn approx_eq(a: f32, b: f32) {
        const EPSILON: f32 = 0.001;
        if (a - b).abs() > EPSILON {
            panic!("a: {} != b: {}", a, b)
        }
    }
    #[test]
    fn vec_new() {
        let x = 0.;
        let y = 0.;
        let z = 2.;
        let w = 3.;
        let v = Vector4::new(x, y, z, w);

        assert!((v.x() - x).abs() < 0.001);
        assert!((v.y() - y).abs() < 0.001);
        assert!((v.z() - z).abs() < 0.001);
        assert!((v.w() - w).abs() < 0.001);

        assert!((v[0] - x).abs() < 0.001);
        assert!((v[1] - y).abs() < 0.001);
        assert!((v[2] - z).abs() < 0.001);
        assert!((v[3] - w).abs() < 0.001);
    }
    #[test]
    fn vec4_index_mut() {
        let mut v = Vector4::ZERO;
        v[0] = 1.;
        v[1] = 2.;
        v[2] = 3.;
        v[3] = 4.;
        approx_eq(v[0], 1.);
        approx_eq(v[1], 2.);
        approx_eq(v[2], 3.);
        approx_eq(v[3], 4.);
    }
    #[test]
    fn matrix_zero() {
        let zero_matrix = Matrix4::zero();
        for x in 0..4 {
            for y in 0..4 {
                approx_eq(zero_matrix[(x, y)], 0.)
            }
        }
        let mul = Vector4::new(2213.1, 31., 2312., 123.);
        let mul_result = zero_matrix * mul;
        for y in 0..4 {
            approx_eq(mul_result[y], 0.);
        }
    }
    #[test]
    fn matrix_identity() {
        let zero_matrix = Matrix4::identity();
        for x in 0..4 {
            for y in 0..4 {
                if x != y {
                    approx_eq(zero_matrix[(x, y)], 0.)
                } else {
                    approx_eq(zero_matrix[(x, y)], 1.)
                }
            }
        }
        let mul = Vector4::new(2213.1, 31., 2312., 123.);
        let mul_result = zero_matrix * mul;
        for x in 0..4 {
            approx_eq(mul_result[x], mul[x])
        }
    }
    #[test]
    fn matrix_index() {
        let matrix = Matrix4::from_rows([
            Vector4::new(0., 1., 2., 3.),
            Vector4::new(10., 11., 12., 13.),
            Vector4::new(20., 21., 22., 23.),
            Vector4::new(30., 31., 32., 33.),
        ]);
        for y in 0..=3 {
            let v = matrix[y];
            for x in 0..=3 {
                let expected = y as f32 * 10. + x as f32;
                let f = matrix[(y, x)];
                let v_f = v[x];
                approx_eq(f, expected);
                approx_eq(v_f, expected);
            }
        }
    }
    #[test]
    fn vec3_new() {
        let data_list = [[0., 0., 0.], [1., 1., 1.], [1., 2., 3.]];
        for data in data_list {
            let v = Vector3::new(data[0], data[1], data[2]);
            approx_eq(v.x(), data[0]);
            approx_eq(v.y(), data[1]);
            approx_eq(v.z(), data[2]);
        }
    }
    #[test]
    fn translation() {
        let translation = Matrix4::translation(Vector3::new(1., 2., 3.));
        let org = Vector4::new(0., 1., 2., 1.);
        let translated = translation * org;
        approx_eq(translated.x(), 1.);
        approx_eq(translated.y(), 3.);
        approx_eq(translated.z(), 5.);
        approx_eq(translated.w(), 1.);
    }
}
