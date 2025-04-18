#![allow(unused)]

use raytracer as rt;

// structs
use rt::{
    camera::Camera,
    color::Color,
    hittable::{HitRecord, Hittable},
    hittable_list::HittableList,
    ray::Ray,
    sphere::Sphere,
    vec3::Point3,
};

// modules
use rt::{color, hittable, hittable_list, ray, sphere, utils, vec3};

// external crates
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufWriter, Write};

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: i32 = 400;
const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
const SAMPLES_PER_PIXEL: i32 = 100;
const MAX_DEPTH: i32 = 50;

fn hit_sphere(center: Point3, radius: f64, r: &Ray) -> f64 {
    let oc = r.origin() - center;
    let a = r.direction().length_squared();
    let half_b = vec3::dot(oc, r.direction());
    let c = oc.length_squared() - radius * radius;
    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        -1.0
    } else {
        (-half_b - f64::sqrt(discriminant)) / a
    }
}

fn ray_color(r: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec = HitRecord::new();
    if world.hit(r, 0.001, f64::INFINITY, &mut rec) {
        let direction = rec.normal + vec3::random_unit_vector();
        return 0.5 * ray_color(&Ray::new(rec.p, direction), world, depth - 1);
    }

    let unit_direction = vec3::unit_vector(r.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn main() -> std::io::Result<()> {
    let file = File::create("image.ppm")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "P3\n{} {}\n255", IMAGE_WIDTH, IMAGE_HEIGHT)?;

    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%)")
            .unwrap()
            .progress_chars("==> "),
    );

    // World
    let mut world = HittableList::new();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // Camera
    let camera = Camera::new();

    // Render
    for j in (0..IMAGE_HEIGHT).rev() {
        bar.inc(1);

        for i in 0..IMAGE_WIDTH {
            let mut pixel_color = Color::new(0.0, 0.0, 0.0);
            for _ in 0..SAMPLES_PER_PIXEL {
                let u = (i as f64 + utils::random_double()) / (IMAGE_WIDTH - 1) as f64;
                let v = (j as f64 + utils::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                let r = camera.get_ray(u, v);
                pixel_color += ray_color(&r, &world, MAX_DEPTH);
            }
            color::write_color(&mut writer, pixel_color, SAMPLES_PER_PIXEL);
        }
    }

    bar.finish_with_message("Image written!");

    Ok(())
}
