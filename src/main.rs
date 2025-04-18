use raytracer as rt;

use rt::{
    camera::Camera,
    color::Color,
    hittable::Hittable,
    hittable_list::HittableList,
    material::{Dielectric, Lambertian, Metal},
    ray::Ray,
    sphere::Sphere,
    vec3::Point3,
};

use rt::{color, utils, vec3};

use indicatif::{ProgressBar, ProgressStyle};
use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

const ASPECT_RATIO: f64 = 16.0 / 9.0;
const IMAGE_WIDTH: i32 = 800;
const IMAGE_HEIGHT: i32 = (IMAGE_WIDTH as f64 / ASPECT_RATIO) as i32;
const SAMPLES_PER_PIXEL: i32 = 100;
const MAX_DEPTH: i32 = 50;
const BUFFER_SIZE: usize = (IMAGE_WIDTH * IMAGE_HEIGHT) as usize;

const DISPLAY_IN_WINDOW: bool = true;

fn ray_color(r: &Ray, world: &dyn Hittable, depth: i32) -> Color {
    if depth <= 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    if let Some(hit_rec) = world.hit(r, 0.001, f64::INFINITY) {
        if let Some(scatter_rec) = hit_rec.mat.scatter(r, &hit_rec) {
            return scatter_rec.attenuation * ray_color(&scatter_rec.scattered, world, depth - 1);
        }
        return Color::new(0.0, 0.0, 0.0);
    }

    let unit_direction = vec3::unit_vector(r.direction());
    let t = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - t) * Color::new(1.0, 1.0, 1.0) + t * Color::new(0.5, 0.7, 1.0)
}

fn random_scene() -> HittableList {
    let mut world = HittableList::new();

    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = utils::random_double();
            let center = Point3::new(
                a as f64 + 0.9 * utils::random_double(),
                0.2,
                b as f64 + 0.9 * utils::random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // Diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    // Metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = utils::random_double_range(0.0, 0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                } else {
                    // Glass
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Box::new(Sphere::new(center, 0.2, sphere_material)));
                }
            }
        }
    }

    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    world
}

fn color_to_u32(color: Color, samples_per_pixel: i32) -> u32 {
    let scale = 1.0 / samples_per_pixel as f64;
    let r = (color.x() * scale).sqrt().clamp(0.0, 0.999);
    let g = (color.y() * scale).sqrt().clamp(0.0, 0.999);
    let b = (color.z() * scale).sqrt().clamp(0.0, 0.999);

    let ir = (256.0 * r) as u32;
    let ig = (256.0 * g) as u32;
    let ib = (256.0 * b) as u32;

    (ir << 16) | (ig << 8) | ib
}

fn render_to_window() {
    let mut buffer: Vec<u32> = vec![0; BUFFER_SIZE];
    let mut window = Window::new(
        "Ray Tracer - Rendering...",
        IMAGE_WIDTH as usize,
        IMAGE_HEIGHT as usize,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("Unable to open window: {}", e);
    });

    let world = random_scene();
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Point3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
    );

    for j in (0..IMAGE_HEIGHT).rev() {
        let pixel_colors: Vec<_> = (0..IMAGE_WIDTH)
            .into_par_iter()
            .map(|i| {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..SAMPLES_PER_PIXEL {
                    let u = (i as f64 + utils::random_double()) / (IMAGE_WIDTH - 1) as f64;
                    let v = (j as f64 + utils::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                    let r = camera.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, MAX_DEPTH);
                }
                pixel_color
            })
            .collect();

        for (i, color) in pixel_colors.into_iter().enumerate() {
            let index = (IMAGE_HEIGHT - 1 - j) * IMAGE_WIDTH + i as i32;
            buffer[index as usize] = color_to_u32(color, SAMPLES_PER_PIXEL);
        }

        window
            .update_with_buffer(&buffer, IMAGE_WIDTH as usize, IMAGE_HEIGHT as usize)
            .unwrap();

        if window.is_key_down(Key::Escape) {
            break;
        }
    }

    window.set_title("Rendering complete! (Press ESC to exit)");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn render_to_file() -> std::io::Result<()> {
    let file = File::create("image.ppm")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "P3\n{} {}\n255", IMAGE_WIDTH, IMAGE_HEIGHT)?;

    let bar = ProgressBar::new(IMAGE_HEIGHT as u64);
    bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({percent}%)")
            .unwrap()
            .progress_chars("=> "),
    );

    let world = random_scene();

    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Point3::new(0.0, 1.0, 0.0);
    let dist_to_focus = 10.0;
    let aperture = 0.1;

    let camera = Camera::new(
        lookfrom,
        lookat,
        vup,
        20.0,
        ASPECT_RATIO,
        aperture,
        dist_to_focus,
    );

    for j in (0..IMAGE_HEIGHT).rev() {
        bar.inc(1);

        let pixel_colors: Vec<_> = (0..IMAGE_WIDTH)
            .into_par_iter()
            .map(|i| {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);

                for _ in 0..SAMPLES_PER_PIXEL {
                    let u = ((i as f64) + utils::random_double()) / (IMAGE_WIDTH - 1) as f64;
                    let v = ((j as f64) + utils::random_double()) / (IMAGE_HEIGHT - 1) as f64;
                    let r = camera.get_ray(u, v);
                    pixel_color += ray_color(&r, &world, MAX_DEPTH);
                }

                pixel_color
            })
            .collect();

        for pixel_color in pixel_colors {
            color::write_color(&mut writer, pixel_color, SAMPLES_PER_PIXEL);
        }
    }

    bar.finish_with_message("Image written!");

    Ok(())
}

fn main() -> std::io::Result<()> {
    if DISPLAY_IN_WINDOW {
        render_to_window();
    } else {
        render_to_file()?;
    }

    Ok(())
}
