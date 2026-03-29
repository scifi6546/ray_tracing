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
    pub fn dot(&self, rhs: Self) -> f32 {
        self.x() * rhs.x() + self.y() * rhs.y() + self.z() * rhs.z() + self.w() * rhs.w()
    }
}
impl Mul<Vector4> for f32 {
    type Output = Vector4;
    fn mul(self, rhs: Vector4) -> Self::Output {
        Vector4::new(
            self * rhs.x(),
            self * rhs.y(),
            self * rhs.z(),
            self * rhs.w(),
        )
    }
}
impl Mul<Matrix4> for f32 {
    type Output = Matrix4;
    fn mul(self, rhs: Matrix4) -> Self::Output {
        Matrix4 {
            rows: [
                self * rhs.rows[0],
                self * rhs.rows[1],
                self * rhs.rows[2],
                self * rhs.rows[3],
            ],
        }
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
        let mut rows = [Vector4::ZERO; 4];
        for row in 0..=3 {
            for col in 0..=3 {
                let l_vector = self.rows[row];
                let r_vector =
                    Vector4::new(rhs[(0, col)], rhs[(1, col)], rhs[(2, col)], rhs[(3, col)]);
                let value = l_vector.dot(r_vector);
                rows[row][col] = value;
            }
        }

        Self { rows }
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
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vector3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub const fn x(&self) -> f32 {
        self.x
    }
    pub const fn y(&self) -> f32 {
        self.y
    }
    pub const fn z(&self) -> f32 {
        self.z
    }
}
#[derive(Clone, Copy)]
pub struct Rotation {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
}
impl Rotation {
    pub fn new(roll: f32, pitch: f32, yaw: f32) -> Self {
        Self { roll, pitch, yaw }
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
    // using https://en.wikipedia.org/wiki/Rotation_matrix#Basic_3D_rotations
    pub fn rotation(rotation: Rotation) -> Self {
        let roll_mat = Matrix4 {
            rows: [
                Vector4::new(rotation.roll.cos(), -rotation.roll.sin(), 0., 0.),
                Vector4::new(rotation.roll.sin(), rotation.roll.cos(), 0., 0.),
                Vector4::new(0., 0., 1., 0.),
                Vector4::new(0., 0., 0., 1.),
            ],
        };
        let yaw_mat = Matrix4 {
            rows: [
                Vector4::new(rotation.yaw.cos(), 0., rotation.yaw.sin(), 0.),
                Vector4::new(0., 1., 0., 0.),
                Vector4::new(-rotation.yaw.sin(), 0., rotation.yaw.cos(), 0.),
                Vector4::new(0., 0., 0., 1.),
            ],
        };
        let pitch_mat = Matrix4 {
            rows: [
                Vector4::new(1., 0., 0., 0.),
                Vector4::new(0., rotation.pitch.cos(), -rotation.pitch.sin(), 0.),
                Vector4::new(0., rotation.pitch.sin(), rotation.pitch.cos(), 0.),
                Vector4::new(0., 0., 0., 1.),
            ],
        };
        // println!("todo: roll and pitch");
        roll_mat * pitch_mat * yaw_mat
    }
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                std::ptr::from_ref(self) as *const u8,
                std::mem::size_of::<Self>(),
            )
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

    fn mat_approx_eq(a: Matrix4, b: Matrix4) {
        for i in 0..=3 {
            for j in 0..=3 {
                let a_val = a[(i, j)];
                let b_val = b[(i, j)];
                approx_eq(a_val, b_val);
            }
        }
    }
    fn vec4_approx_eq(a: Vector4, b: Vector4) {
        for i in 0..=3 {
            approx_eq(a[i], b[i]);
        }
    }
    #[test]
    fn vec4_new() {
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
    fn scalar_vec4_mul() {
        let s = 0.;
        let v = Vector4::ZERO;

        vec4_approx_eq(s * v, v);
        let v = Vector4::new(1., 2., 3., 4.);
        vec4_approx_eq(2. * v, Vector4::new(2., 4., 6., 8.));
    }
    #[test]
    fn scalar_mat4_mul() {
        let m = Matrix4::from_rows([
            Vector4::new(0., 1., 2., 3.),
            Vector4::new(4., 5., 6., 7.),
            Vector4::new(8., 9., 10., 11.),
            Vector4::new(12., 13., 14., 15.),
        ]);
        let r = Matrix4::from_rows([
            Vector4::new(2. * 0., 2. * 1., 2. * 2., 2. * 3.),
            Vector4::new(2. * 4., 2. * 5., 2. * 6., 2. * 7.),
            Vector4::new(2. * 8., 2. * 9., 2. * 10., 2. * 11.),
            Vector4::new(2. * 12., 2. * 13., 2. * 14., 2. * 15.),
        ]);
        mat_approx_eq(2. * m, r);
    }
    #[test]
    fn vec4_dot() {
        let zero = Vector4::ZERO;

        approx_eq(zero.dot(zero), 0.);
        let one = Vector4::new(1., 0., 0., 0.);
        approx_eq(one.dot(one), 1.);

        let one = Vector4::new(0., 1., 0., 0.);
        approx_eq(one.dot(one), 1.);

        let one = Vector4::new(0., 0., 1., 0.);
        approx_eq(one.dot(one), 1.);

        let one = Vector4::new(0., 0., 0., 1.);
        approx_eq(one.dot(one), 1.);
    }
    #[test]
    fn mul_mat4() {
        let rotation = Matrix4::rotation(Rotation::new(1., 23., 321.));
        let identity = Matrix4::identity();
        let zero = Matrix4::zero();

        mat_approx_eq(identity * identity, identity);
        mat_approx_eq(identity * rotation, rotation);
        mat_approx_eq(rotation * identity, rotation);
        mat_approx_eq(zero * identity, zero);
        mat_approx_eq(identity * zero, zero);

        mat_approx_eq(zero * rotation, zero);
        mat_approx_eq(rotation * zero * rotation, zero);
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
        assert_eq!(size_of::<Vector3>(), 3 * size_of::<f32>());
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
    #[test]
    fn as_bytes() {
        let identity = Matrix4::identity();
        let bytes = identity.as_bytes();
        assert_eq!(bytes.len(), 16 * size_of::<f32>());
        unsafe {
            let read_mat = std::ptr::read_volatile(bytes.as_ptr() as *const Matrix4);
            mat_approx_eq(identity, read_mat);
        }
    }
}
