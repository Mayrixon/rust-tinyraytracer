use std::fs::*;
use std::io::{prelude::*, BufWriter};
use vek;

fn render() {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;
    let mut framebuffer = vec![vec![vek::Rgb::<f64>::zero(); WIDTH]; HEIGHT];

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            framebuffer[j][i] =
                vek::Rgb::new(j as f64 / HEIGHT as f64, i as f64 / WIDTH as f64, 0.);
        }
    }

    let file = File::create("./target/out.ppm").unwrap();
    let mut buffer = BufWriter::new(file);
    buffer
        .write_fmt(format_args!("P6\n{} {}\n255\n", WIDTH, HEIGHT))
        .unwrap();

    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            for k in 0..3 {
                let a = (255. * (framebuffer[i][j][k]).min(1.).max(0.)) as u8;
                buffer.write(&[a]).unwrap();
            }
        }
    }
    buffer.flush().unwrap();
}

fn main() {
    render();
}
