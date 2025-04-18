#![allow(unused)]

use std::io::Write;

use crate::utils;
use crate::vec3::Vec3;

pub type Color = Vec3;

pub fn write_color(out: &mut impl Write, pixel_color: Color, samples_per_pixel: i32) {
    let mut r = pixel_color.x();
    let mut g = pixel_color.y();
    let mut b = pixel_color.z();

    let scale = 1.0 / samples_per_pixel as f64;
    r = f64::sqrt(scale * r);
    g = f64::sqrt(scale * g);
    b = f64::sqrt(scale * b);

    writeln!(
        out,
        "{} {} {}",
        (256.0 * utils::clamp(r, 0.0, 0.999)) as i32,
        (256.0 * utils::clamp(g, 0.0, 0.999)) as i32,
        (256.0 * utils::clamp(b, 0.0, 0.999)) as i32,
    )
    .expect("writing color");
}
