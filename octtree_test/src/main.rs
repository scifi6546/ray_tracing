use cgmath::{Point3, Vector3};
use image::Rgb;
use lib_minya::{prelude::Ray, ray_tracer::hittable::OctTree};

fn main() {
    let oct_tree = OctTree::sphere(10, ());
    //let oct_tree = OctTree::cube(4, ());
    let x_size = 200;
    let y_size = 200;
    let origin = Point3::new(15.0f32, 10.0, -50.0);
    let focal_length = 4.0;
    let width = 2.0f32;
    let height = 2.0f32;
    let viewport_u = Vector3::new(width, 0.0, 0.0);
    let viewport_v = Vector3::new(0.0, -height, 0.0);
    let pixel_delta_u = viewport_u / x_size as f32;
    let pixel_delta_v = viewport_v / y_size as f32;
    let viewport_upper_left =
        origin - Vector3::new(0.0, 0.0, -focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);
    let mut img = image::RgbImage::new(x_size, y_size);
    for x in 0..x_size {
        for y in 0..y_size {
            let debug_pixel = x == 3 && y == 0;
            if debug_pixel {
                println!("debug pixel");
            }
            let pixel_center =
                pixel00_loc + (x as f32 * pixel_delta_u) + (y as f32 * pixel_delta_v);
            let ray_direction = pixel_center - origin;
            let ray = Ray {
                origin,
                direction: ray_direction,
                time: 0.,
            };

            if let Some(hit_info) = oct_tree.trace_ray(ray) {
                if debug_pixel {
                    println!("hit info: {:#?}", hit_info)
                }
                //println!("hit info: {:#?}", hit_info);
                img.put_pixel(x, y, Rgb([255, 255, 0]));
            }
        }
    }
    img.save("sphere.png").expect("failed to save");
    println!("Hello, world!");
}
