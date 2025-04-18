use indicatif::{ProgressBar, ProgressStyle};
use std::fs::File;
use std::io::{BufWriter, Write};

const IMAGE_WIDTH: i32 = 256;
const IMAGE_HEIGHT: i32 = 256;

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

    for j in (0..IMAGE_HEIGHT).rev() {
        bar.inc(1);

        for i in 0..IMAGE_WIDTH {
            let r = i as f64 / (IMAGE_WIDTH - 1) as f64;
            let g = j as f64 / (IMAGE_HEIGHT - 1) as f64;
            let b = 0.25;

            let ir = (255.999 * r) as i32;
            let ig = (255.999 * g) as i32;
            let ib = (255.999 * b) as i32;

            writeln!(writer, "{} {} {}", ir, ig, ib)?;
        }
    }

    bar.finish_with_message("Image written!");

    Ok(())
}
