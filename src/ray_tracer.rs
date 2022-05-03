use super::Image;
use cgmath::Vector3;
use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread,
    time::Duration,
};
struct Ray {
    origin: Vector3<f32>,
    direction: Vector3<f32>,
}
pub struct RayTracer {
    sender: Sender<Image>,
}
impl RayTracer {
    pub fn new() -> Receiver<Image> {
        let (sender, recvier) = channel();
        let s = Self { sender };
        thread::spawn(move || s.start_tracing());
        recvier
    }
    pub fn start_tracing(&self) {
        self.sender
            .send(Image::from_fn(|x, y| [0, 0, 0, 0xff], 1000, 1000))
            .expect("failed to send");
        let mut i = 0;
        loop {
            thread::sleep(Duration::from_millis(100));
            let mut img = Image::from_fn(|_, _| [0, 0, 0, 0xff], 1000, 1000);
            for x in 0..1000 {
                for y in 0..1000 {
                    let brightness = (((i % 10u32) * 30u32) & 0xff) as u8;
                    img.set_xy(x, y, [brightness, brightness, brightness, 0xff]);
                }
            }
            self.sender.send(img).expect("failed to send");
            i += 1;
        }
    }
}
