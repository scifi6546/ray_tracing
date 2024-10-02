use cgmath::Point2;

use crate::{
    prelude::*,
    ray_tracer::{ray_tracer_info::EntityField, RayTracer},
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

#[derive(Clone)]
pub struct ParallelImage {
    buffer: Vec<RgbColor>,
    width: usize,
    height: usize,
}
impl ParallelImage {
    pub fn from_buffer(buffer: Vec<RgbColor>, width: usize, height: usize) -> Self {
        assert_eq!(width * height, buffer.len());
        Self {
            buffer,
            width,
            height,
        }
    }

    pub fn to_image(&self, num_samples: usize) -> image::RgbImage {
        let normalized_buffer: Vec<RgbColor> = self
            .buffer
            .iter()
            .map(|c| *c / num_samples as f32)
            .collect();
        let mi = normalized_buffer
            .iter()
            .flat_map(|p| [p.red, p.green, p.blue])
            .fold(f32::MAX, |acc, x| acc.min(x));
        let ma = normalized_buffer
            .iter()
            .flat_map(|p| [p.red, p.green, p.blue])
            .fold(f32::MAX, |acc, x| acc.max(x));
        info!("image min: {}, image max: {}", mi, ma);
        let mut image = image::RgbImage::from_pixel(
            self.width as u32,
            self.height as u32,
            image::Rgb([0, 0, 0]),
        );
        for x in 0..self.width() {
            for y in 0..self.height() {
                let color = self.get_xy(x, y) / num_samples as f32;

                let rgb = color.as_rgb_u8();
                let u8_color = image::Rgb(rgb);
                image.put_pixel(x as u32, self.height() as u32 - y as u32 - 1, u8_color);
            }
        }
        return image;
    }
    pub fn get_uv(&self, uv: Point2<RayScalar>) -> RgbColor {
        let x = ((uv.x * (self.width() as RayScalar - 1.0)) as usize).clamp(0, self.width() - 1);
        let v = 1.0 - uv.y;
        let y = ((v * (self.height() as RayScalar - 1.0)) as usize).clamp(0, self.height() - 1);
        self.get_xy(x, y)
    }
    pub fn save_image<P: AsRef<Path>>(&self, p: P, num_samples: usize) {
        let img = self.to_image(num_samples);
        img.save(p).expect("failed to save image");
    }

    pub fn new_black(width: usize, height: usize) -> Self {
        Self {
            buffer: vec![RgbColor::BLACK; width * height],
            width,
            height,
        }
    }

    fn get_idx_no_self(width: usize, x: usize, y: usize) -> usize {
        x + y * width
    }
    fn get_idx(&self, x: usize, y: usize) -> usize {
        Self::get_idx_no_self(self.width, x, y)
    }
    pub fn get_clamped(&self, x: i32, y: i32) -> RgbColor {
        self.get_xy(
            x.max(0).min(self.width as i32 - 1) as usize,
            y.max(0).min(self.height as i32 - 1) as usize,
        )
    }
    pub(crate) fn height(&self) -> usize {
        self.height
    }
    pub(crate) fn width(&self) -> usize {
        self.width
    }
    pub(crate) fn split(&self, num_parts: usize) -> Vec<ParallelImagePart> {
        let slice_width = self.width() / num_parts;
        let mut out_images = vec![];
        out_images.reserve(num_parts);
        for slice in 0..num_parts {
            let slice_start = slice_width * slice;
            let slice_end = (slice_start + slice_width).min(self.width());

            let mut buffer = vec![];
            buffer.reserve((slice_end - slice_start) * self.height);

            for y in 0..self.height {
                for x in slice_start..slice_end {
                    buffer.push(self.buffer[self.get_idx(x, y)]);
                }
            }
            out_images.push(ParallelImagePart {
                buffer,
                width: slice_end - slice_start,
                height: self.height(),
                total_width: self.width,
                offset: Point2::new(slice_start, 0),
            });
        }
        out_images
    }
    fn join_container(mut images: Vec<&PartContainer>) -> Self {
        assert!(!images.is_empty());
        images.sort_by(|img1, img2| {
            img1.image
                .offset
                .x
                .partial_cmp(&img2.image.offset.x)
                .unwrap()
        });
        let last = &images.last().as_ref().unwrap().image;
        let width = last.offset.x + last.width;
        let mut buffer = vec![RgbColor::BLACK; width * last.height];
        for img in images.iter() {
            let image = &img.image;
            for x in 0..image.width {
                for y in 0..image.height {
                    let idx = Self::get_idx_no_self(width, x + img.image.offset.x, y);
                    buffer[idx] = image.get_xy(x + img.image.offset.x, y + img.image.offset.y)
                        / img.num_samples as f32;
                }
            }
        }
        Self {
            buffer,
            width,
            height: last.height,
        }
    }
    pub(crate) fn join(mut images: Vec<&ParallelImagePart>) -> Self {
        assert!(!images.is_empty());
        images.sort_by(|img1, img2| img1.offset.x.partial_cmp(&img2.offset.x).unwrap());
        let last = images.last().unwrap();
        let width = last.offset.x + last.width;
        let mut buffer = vec![RgbColor::BLACK; width * last.height];
        for img in images.iter() {
            for x in 0..img.width {
                for y in 0..img.height {
                    let idx = Self::get_idx_no_self(width, x + img.offset.x, y);
                    buffer[idx] = img.get_xy(x + img.offset.x, y + img.offset.y);
                }
            }
        }
        Self {
            buffer,
            width,
            height: last.height,
        }
    }
    pub fn get_xy(&self, x: usize, y: usize) -> RgbColor {
        self.buffer[self.get_idx(x, y)]
    }
    pub fn set_xy(&mut self, x: usize, y: usize, color: RgbColor) {
        //self.buffer[todo!()] = color;
        let idx = self.get_idx(x, y);
        self.buffer[idx] = color;
    }
    pub fn filter_nan(&mut self, replacement: RgbColor) {
        for x in 0..self.width {
            for y in 0..self.height {
                let val = self.get_xy(x, y);
                if val.is_nan() {
                    self.set_xy(x, y, replacement)
                }
            }
        }
    }
    pub fn down_sample(&self) -> Self {
        let new_width = self.width() / 2;
        let new_height = self.height() / 2;
        let mut img = Self::new_black(new_width, new_height);

        for y in 0..new_height {
            for x in 0..new_width {
                let x0y0 = self.get_xy(x * 2, y * 2);
                let x1y0 = self.get_xy((x * 2 + 1).min(self.width() - 1), y * 2);
                let x1y1 = self.get_xy(
                    (x * 2 + 1).min(self.width() - 1),
                    (y * 2 + 1).min(self.height() - 1),
                );
                let x0y1 = self.get_xy(x * 2, (y * 2 + 1).min(self.height() - 1));
                let avg = (x0y0 + x1y0 + x1y1 + x0y1) / 4.0;

                img.set_xy(x, y, avg);
            }
        }
        img
    }
    /// Gets nearest neighbor x: [0,1] y: [0,1]
    pub fn get_nearest(&self, x: f32, y: f32) -> RgbColor {
        assert!(self.width() > 1);
        assert!(self.height() > 1);
        let x_pixel = (x * (self.width() - 1) as f32)
            .min(self.width() as f32 - 1.0)
            .max(0.0);
        let y_pixel = (y * (self.height() - 1) as f32)
            .min(self.height() as f32 - 1.0)
            .max(0.0);
        let x0 = x_pixel.floor() as usize;
        let y0 = y_pixel.floor() as usize;

        let x1 = (x0 + 1).min(self.width() - 1);
        let y1 = (y0 + 1).min(self.height() - 1);
        let x0y0 = self.get_xy(x0, y0);
        let x1y0 = self.get_xy(x1, y0);
        let x1y1 = self.get_xy(x1, y1);
        let x0y1 = self.get_xy(x0, y1);

        let x_fract = x_pixel.fract();
        let y_fract = y_pixel.fract();

        let x0 = (1.0 - y_fract) * x0y0 + y_fract * x0y1;
        let x1 = (1.0 - y_fract) * x1y0 + y_fract * x1y1;
        (1.0 - x_fract) * x0 + x_fract * x1
    }
    pub fn add_xy(&mut self, x: usize, y: usize, color: RgbColor) {
        let idx = self.get_idx(x, y);
        self.buffer[idx] += color;
    }
}
impl std::ops::Add<&ParallelImage> for ParallelImage {
    type Output = Self;

    fn add(mut self, rhs: &ParallelImage) -> Self::Output {
        for y in 0..self.height() {
            let y_f = (y as f32 + 0.5) / (self.height() as f32 - 1.0);
            for x in 0..self.width() {
                let x_f = (x as f32 + 0.5) / (self.width() as f32 - 1.0);
                let pixel = rhs.get_nearest(x_f, y_f);
                self.add_xy(x, y, pixel)
            }
        }
        self
    }
}
impl std::ops::Add<ParallelImage> for ParallelImage {
    type Output = Self;

    fn add(self, rhs: ParallelImage) -> Self::Output {
        self.add(&rhs)
    }
}
impl std::ops::Div<f32> for ParallelImage {
    type Output = ParallelImage;

    fn div(mut self, rhs: f32) -> Self::Output {
        Self {
            buffer: self.buffer.drain(..).map(|c| c / rhs).collect(),
            width: self.width,
            height: self.height,
        }
    }
}
#[derive(Clone)]
pub struct ParallelImagePart {
    buffer: Vec<RgbColor>,
    width: usize,
    height: usize,
    total_width: usize,
    offset: Point2<usize>,
}
impl ParallelImagePart {
    pub(crate) fn width(&self) -> usize {
        self.width
    }
    pub(crate) fn height(&self) -> usize {
        self.height
    }
    pub(crate) fn offset(&self) -> Point2<usize> {
        self.offset
    }
    fn get_idx(&self, x: usize, y: usize) -> usize {
        assert!(x >= self.offset.x);
        if x >= self.offset.x + self.width {
            info!(
                "x: {}, self.offset.x: {}, self.width: {}",
                x, self.offset.x, self.width
            );
        }

        assert!(x < self.offset.x + self.width);

        assert!(y < self.height);
        (x - self.offset.x) + y * self.width
    }
    pub(crate) fn set_black(&mut self) {
        for pixel in self.buffer.iter_mut() {
            *pixel = RgbColor::BLACK;
        }
    }
    /// gets with offset
    pub(crate) fn get_xy(&self, x: usize, y: usize) -> RgbColor {
        self.buffer[self.get_idx(x, y)]
    }
    pub(crate) fn total_width(&self) -> usize {
        self.total_width
    }
    pub(crate) fn total_height(&self) -> usize {
        self.height
    }

    ///adds with offset
    pub fn add_xy(&mut self, x: usize, y: usize, color: RgbColor) {
        let idx = self.get_idx(x, y);
        self.buffer[idx] += color;
    }
}

pub(crate) struct ImageReceiver {
    receiver: Receiver<ParallelImagePart>,
    num_items: Arc<Mutex<usize>>,
}
impl ImageSender {
    const MAX_ITEMS: usize = 10;
    pub(crate) fn send(&mut self, image: ParallelImagePart) {
        let mut num_items = self.num_items.lock().expect("failed to get num items");
        if *num_items < Self::MAX_ITEMS {
            self.sender.send(image).expect("failed to send image");
            *num_items += 1;
        }
    }
}
pub(crate) struct ImageSender {
    sender: Sender<ParallelImagePart>,
    num_items: Arc<Mutex<usize>>,
}
pub(crate) fn image_channel() -> (ImageSender, ImageReceiver) {
    let (sender, receiver) = channel();
    let num_items = Arc::new(Mutex::new(0));
    (
        ImageSender {
            sender,
            num_items: num_items.clone(),
        },
        ImageReceiver {
            receiver,
            num_items,
        },
    )
}

impl ImageReceiver {
    pub(crate) fn try_recv(&mut self) -> Option<ParallelImagePart> {
        let mut num_items = self.num_items.lock().expect("failed to get num items");
        let parallel_image_opt = self.receiver.try_recv();
        match parallel_image_opt {
            Ok(img) => {
                *num_items -= 1;
                Some(img)
            }
            Err(e) => match e {
                std::sync::mpsc::TryRecvError::Disconnected => panic!("channel disconnected"),
                std::sync::mpsc::TryRecvError::Empty => None,
            },
        }
    }
}
pub(crate) enum RayTracerMessage {
    LoadScenario(String),
    #[allow(dead_code)]
    SetShader(super::ray_tracer::CurrentShader),
    StopRendering,
    ContinueRendering,
    SetCameraData((String, EntityField)),
}
struct PartContainer {
    image: ParallelImagePart,
    num_samples: usize,
}
pub struct ParallelImageCollector {
    receivers: Vec<ImageReceiver>,
    message_senders: Vec<Sender<RayTracerMessage>>,
    images: HashMap<Point2<usize>, PartContainer>,
    ray_tracer: Arc<RwLock<RayTracer>>,
}
impl ParallelImageCollector {
    pub(crate) fn new(
        receivers: Vec<ImageReceiver>,
        message_senders: Vec<Sender<RayTracerMessage>>,
        ray_tracer: Arc<RwLock<RayTracer>>,
    ) -> Self {
        Self {
            receivers,
            message_senders,
            images: HashMap::new(),
            ray_tracer,
        }
    }
    /// resets rendering image to base state
    pub(crate) fn clear(&mut self) {
        self.images.clear();
    }
    pub fn receive(&mut self) -> Option<ParallelImage> {
        for recv in self.receivers.iter_mut() {
            loop {
                if let Some(image) = recv.try_recv() {
                    let num_samples = self
                        .images
                        .get(&image.offset())
                        .map(|img| img.num_samples + 1)
                        .unwrap_or(1);

                    self.images
                        .insert(image.offset, PartContainer { image, num_samples });
                } else {
                    break;
                }
            }
        }
        if self.images.len() >= 1 {
            Some(ParallelImage::join_container(
                self.images.iter().map(|(_k, v)| v).collect(),
            ))
        } else {
            None
        }
    }
    pub fn load_scenario(&mut self, name: String) {
        self.message_senders.iter_mut().for_each(|s| {
            s.send(RayTracerMessage::StopRendering)
                .map_err(|e| error!("failed to send stop rendering message, reason: {}", e))
                .unwrap();
        });
        info!("loading scenario");
        self.clear();

        {
            let mut write_lock = self.ray_tracer.write().expect("failed to read");
            write_lock.load_scenario(name.clone());
        }
        for sender in self.message_senders.iter() {
            sender
                .send(RayTracerMessage::LoadScenario(name.clone()))
                .expect("failed to send")
        }
        self.message_senders.iter_mut().for_each(|s| {
            s.send(RayTracerMessage::ContinueRendering)
                .map_err(|e| error!("failed to send continue rendering message, reason: {}", e))
                .unwrap();
        });
    }
    pub fn save_file<P: AsRef<Path>>(&mut self, p: P) {
        if let Some(img) = self.receive() {
            info!("saving image to: {:?}", p.as_ref());
            let default_extension = "png";
            let save_path = if p.as_ref().extension().is_some() {
                p.as_ref().to_path_buf()
            } else {
                p.as_ref().with_extension(default_extension)
            };
            img.save_image(save_path, 1)
        } else {
            error!("no images yet")
        }
    }
    pub fn set_shader(&mut self, s: super::ray_tracer::CurrentShader) {
        {
            let mut write_lock = self.ray_tracer.write().expect("failed to read");
            write_lock.set_shader(s);
        }
        for sender in self.message_senders.iter() {
            sender
                .send(RayTracerMessage::SetShader(s))
                .expect("failed to send shader")
        }
    }
    pub fn set_camera_data(&mut self, key: String, value: EntityField) {
        self.message_senders.iter_mut().for_each(|s| {
            s.send(RayTracerMessage::StopRendering)
                .map_err(|e| error!("failed to send stop rendering message, reason: {}", e))
                .unwrap();
        });
        info!("told ray tracers to stop");
        self.clear();

        {
            let mut write_lock = self.ray_tracer.write().expect("failed to read");
            info!("got write lock for ray tracer now setting data");
            write_lock.set_camera_data(key.clone(), value.clone());
        }
        for sender in self.message_senders.iter() {
            sender
                .send(RayTracerMessage::SetCameraData((
                    key.clone(),
                    value.clone(),
                )))
                .expect("failed to send data");
        }
        self.message_senders.iter_mut().for_each(|s| {
            s.send(RayTracerMessage::ContinueRendering)
                .map_err(|e| error!("failed to send continue rendering message, reason: {}", e))
                .unwrap();
        });
    }
}
