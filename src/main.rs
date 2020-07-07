use std::fs::*;
use std::io::{prelude::*, BufWriter};
use vek;

struct Sphere {
    center: vek::Vec3<f64>,
    radius: f64,
}

impl Sphere {
    fn new(center: vek::Vec3<f64>, radius: f64) -> Self {
        Self { center, radius }
    }

    fn ray_intersect(&self, orig: vek::Vec3<f64>, dir: vek::Vec3<f64>, t0: &mut f64) -> bool {
        let L = self.center - orig;
        let tca = L.dot(dir);
        let d2 = L.dot(L) - tca * tca;
        if d2 > self.radius * self.radius {
            return false;
        } else {
            let thc = (self.radius * self.radius - d2).sqrt();
            *t0 = tca - thc;
            let t1 = tca + thc;
            if *t0 < 0. {
                *t0 = t1;
                return false;
            } else {
                return true;
            }
        }
    }
}

fn cast_ray(orig: vek::Vec3<f64>, dir: vek::Vec3<f64>, sphere: &Sphere) -> vek::Rgb<f64> {
    let mut sphere_dist = std::f64::MAX;
    if !sphere.ray_intersect(orig, dir, &mut sphere_dist) {
        return vek::Rgb::new(0.2, 0.7, 0.8);
    } else {
        return vek::Rgb::new(0.4, 0.4, 0.3);
    }
}

fn render(sphere: &Sphere) {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;
    const FOV: usize = std::f64::consts::FRAC_PI_2 as usize;
    let mut framebuffer = vec![vec![vek::Rgb::<f64>::zero(); WIDTH]; HEIGHT];

    for j in 0..HEIGHT {
        for i in 0..WIDTH {
            let x = (2. * (i as f64 + 0.5) / WIDTH as f64 - 1.)
                * (FOV as f64 / 2.).tan()
                * WIDTH as f64
                / HEIGHT as f64;
            let y = -(2. * (j as f64 + 0.5) / HEIGHT as f64 - 1.) * (FOV as f64 / 2.).tan();
            let dir = vek::Vec3::new(x, y, -1.).normalized();
            framebuffer[j][i] = cast_ray(vek::Vec3::zero(), dir, sphere);
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
    let sphere = Sphere::new(vek::Vec3::new(-3., 0., -16.), 2.);
    render(&sphere);
}
