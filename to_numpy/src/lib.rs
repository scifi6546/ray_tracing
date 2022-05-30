use std::{fs::File, io::Write, path::Path};

pub trait NumpyArray2D {
    fn get(&self, item: [usize; 2]) -> f32;
    fn shape(&self) -> [usize; 2];
    fn get_numpy_data(&self) -> Vec<u8> {
        let shape = self.shape();

        let header_str = format!(
            "{{'descr': '<f4', 'fortran_order': False, 'shape': ({}, {}), }}",
            shape[0], shape[1]
        );
        let header_bytes = header_str.as_bytes();
        let header_len = header_bytes.len() as u32;
        let mut out_data = vec![
            0x93, 'N' as u8, 'U' as u8, 'M' as u8, 'P' as u8, 'Y' as u8, 0x3, 0x00,
        ];
        for byte in header_len.to_le_bytes().iter() {
            out_data.push(*byte);
        }
        for byte in header_bytes.iter() {
            out_data.push(*byte);
        }

        for x in 0..shape[0] {
            for y in 0..shape[1] {
                let val = self.get([x, y]);
                for byte in val.to_ne_bytes().iter() {
                    out_data.push(*byte);
                }
            }
        }

        return out_data;
    }
    fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(&self.get_numpy_data())
    }
}
pub trait NumpyArray3D {
    fn get(&self, item: [usize; 3]) -> f32;
    fn shape(&self) -> [usize; 3];
    fn get_numpy_data(&self) -> Vec<u8> {
        let shape = self.shape();

        let header_str = format!(
            "{{'descr': '<f4', 'fortran_order': False, 'shape': ({}, {}, {}), }}",
            shape[0], shape[1], shape[2]
        );
        let header_bytes = header_str.as_bytes();
        let header_len = header_bytes.len() as u32;
        let mut out_data = vec![
            0x93, 'N' as u8, 'U' as u8, 'M' as u8, 'P' as u8, 'Y' as u8, 0x3, 0x00,
        ];
        for byte in header_len.to_le_bytes().iter() {
            out_data.push(*byte);
        }
        for byte in header_bytes.iter() {
            out_data.push(*byte);
        }

        for x in 0..shape[0] {
            for y in 0..shape[1] {
                for z in 0..shape[2] {
                    let val = self.get([x, y, z]);
                    for byte in val.to_ne_bytes().iter() {
                        out_data.push(*byte);
                    }
                }
            }
        }

        return out_data;
    }
    fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(&self.get_numpy_data())
    }
}

fn iter_dim() {}
#[cfg(test)]
mod tests {
    use super::*;
    struct ZeroMat {
        size: [usize; 2],
    }
    impl NumpyArray2D for ZeroMat {
        fn get(&self, item: [usize; 2]) -> f32 {
            0.0
        }
        fn shape(&self) -> [usize; 2] {
            self.size
        }
    }
    #[test]
    fn two_d() {
        // TODO ACTUAL UNIT TEST
        let mat = ZeroMat { size: [1, 1] };
    }
}
