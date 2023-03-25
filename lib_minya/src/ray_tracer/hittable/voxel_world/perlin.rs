use crate::prelude::*;
use rand::prelude::*;

struct PerlinGrid {
    data: Vec<f32>,
    width: usize,
    height: usize,
}
impl PerlinGrid {
    pub fn new(width: usize, height: usize) -> Self {
        let mut rng = rand::rngs::StdRng::seed_from_u64(1233897876);
        let mut data = vec![];
        data.reserve(width * height);
        for _ in 0..width {
            for _ in 0..height {
                data.push(rng.gen_range(0.0..1.0))
            }
        }
        Self {
            data,
            width,
            height,
        }
    }
    fn get_idx(&self, x: usize, y: usize) -> usize {
        x + self.width * y
    }
    //get in range
    pub fn get(&self, x: f32, y: f32) -> f32 {
        if x >= 0.0 && x <= 1.0 && y >= 0.0 && y <= 1.0 {
            let x_get = x * self.width as f32;
            let x0 = (x_get.floor() as usize).min(self.width - 1);
            let x1 = (x0 + 1).min(self.width - 1);

            let y_get = y * self.height as f32;
            let y0 = (y_get.floor() as usize).min(self.height - 1);
            let y1 = (y0 + 1).min(self.height - 1);

            let rx0_y0 = self.data[self.get_idx(x0, y0)];
            let rx1_y0 = self.data[self.get_idx(x1, y0)];
            let ry0 = (1.0 - x_get.fract()) * rx0_y0 + x_get.fract() * rx1_y0;
            let rx0_y1 = self.data[self.get_idx(x0, y1)];
            let rx1_y1 = self.data[self.get_idx(x1, y1)];
            let ry1 = (1.0 - x_get.fract()) * rx0_y1 + x_get.fract() * rx1_y1;
            (1.0 - y_get.fract()) * ry0 + y_get.fract() * ry1
        } else {
            panic!()
        }
    }
}
pub struct PerlinBuilder {
    size: usize,
    num_layers: usize,
}
impl PerlinBuilder {
    pub fn new(size: usize) -> Self {
        Self {
            size,
            num_layers: (size as f32).log2().floor() as usize,
        }
    }
    pub fn num_layers(self, num_layers: usize) -> Self {
        Self {
            size: self.size,
            num_layers,
        }
    }
    pub fn build(self) -> PerlinNoise {
        PerlinNoise::new(self.size, self.num_layers)
    }
}
pub struct PerlinNoise {
    layers: Vec<PerlinGrid>,
}
impl PerlinNoise {
    pub fn new(size: usize, num_layers: usize) -> Self {
        Self {
            layers: (1..=(size as f32).log2().floor() as usize)
                .rev()
                .enumerate()
                .filter_map(|(idx, res)| if idx < num_layers { Some(res) } else { None })
                .map(|res| {
                    info!("res: {}, res pow: {}", res, 2usize.pow(res as u32));
                    PerlinGrid::new(2usize.pow(res as u32), 2usize.pow(res as u32))
                })
                .collect(),
        }
    }
    pub fn get(&self, x: f32, y: f32) -> f32 {
        self.layers
            .iter()
            .map(|l| l.get(x, y))
            .enumerate()
            .fold(0.0, |acc, (idx, x)| acc + (2.0f32).powi(idx as i32) * x)
    }
}
