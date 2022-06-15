use cgmath::Matrix;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Clone, Copy)]
pub struct Mat4 {
    pub data: [f32; 4 * 4],
}
impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ],
        }
    }
}
pub fn Mat4ToBytes<'a>(mat: &'a cgmath::Matrix4<f32>) -> &'a [u8] {
    let ptr = mat.as_ptr() as *const u8;
    let arr = std::ptr::slice_from_raw_parts(ptr, std::mem::size_of::<f32>() * 4 * 4);
    unsafe { &*arr }
}
