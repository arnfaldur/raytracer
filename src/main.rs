use std::fs::File;
use std::io::Write;

mod vec3;
mod color;

const IMAGE_WIDTH: usize = 512;
const IMAGE_HEIGHT: usize = 512;

fn main() -> std::io::Result<()> {
    let mut file = File::create("image.ppm")?;
    file.write_all(format!("P3\n{} {}\n255\n", IMAGE_WIDTH, IMAGE_HEIGHT).as_bytes())?;

    let pixel_count = IMAGE_WIDTH * IMAGE_HEIGHT;

    let mut dragger = 0.0;

    for j in 0..IMAGE_HEIGHT {
        let nth_pixel = j * IMAGE_WIDTH;
        let progress = nth_pixel as f64 / (pixel_count - 1) as f64;
        if progress - dragger > 0.1 {
            dragger = progress;
            println!("{:.2}%", progress * 100.0);
        }
        for i in 0..IMAGE_WIDTH {
            let color = color::Color::new(
                i as f64 / (IMAGE_WIDTH as f64 - 1.0),
                j as f64 / (IMAGE_HEIGHT as f64 - 1.0),
                0.0,
            );
            color.write_to_file(&mut file)?;
        }
    }
    println!("Done.");
    Ok(())
}
