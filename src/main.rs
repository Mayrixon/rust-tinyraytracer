use std::fs::*;
use std::io::{prelude::*, BufWriter};
use vek;

#[derive(Clone, Copy)]
struct Material {
    diffuse_color: vek::Rgb<f64>,
}

impl Material {
    fn new(color: vek::Rgb<f64>) -> Self {
        Self {
            diffuse_color: color,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self {
            diffuse_color: vek::Rgb::black(),
        }
    }
}

struct Sphere {
    center: vek::Vec3<f64>,
    radius: f64,
    material: Material,
}

impl Sphere {
    fn new(center: vek::Vec3<f64>, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
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

fn scene_intersect(
    orig: vek::Vec3<f64>,
    dir: vek::Vec3<f64>,
    spheres: &Vec<Sphere>,
    hit: &mut vek::Vec3<f64>,
    N: &mut vek::Vec3<f64>,
    material: &mut Material,
) -> bool {
    let mut spheres_dist = std::f64::MAX;
    for i in 0..spheres.len() {
        let mut dist_i = std::f64::NAN;
        if spheres[i].ray_intersect(orig, dir, &mut dist_i) && dist_i < spheres_dist {
            spheres_dist = dist_i;
            *hit = orig + dir * dist_i;
            *N = (*hit - spheres[i].center).normalized();
            *material = spheres[i].material;
        }
    }
    return spheres_dist < 1000.;
}

fn cast_ray(orig: vek::Vec3<f64>, dir: vek::Vec3<f64>, spheres: &Vec<Sphere>) -> vek::Rgb<f64> {
    let mut point = vek::Vec3::<f64>::default();
    let mut N = vek::Vec3::<f64>::default();
    let mut material = Material::default();

    if !scene_intersect(orig, dir, spheres, &mut point, &mut N, &mut material) {
        return vek::Rgb::new(0.2, 0.7, 0.8);
    } else {
        return material.diffuse_color;
    }
}

fn render(spheres: &Vec<Sphere>) {
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
            framebuffer[j][i] = cast_ray(vek::Vec3::zero(), dir, spheres);
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
    let ivory = Material::new(vek::Rgb::new(0.4, 0.4, 0.3));
    let red_rubber = Material::new(vek::Rgb::new(0.3, 0.1, 0.1));

    let mut spheres = Vec::default();
    spheres.push(Sphere::new(vek::Vec3::new(-3., 0., -16.), 2., ivory));
    spheres.push(Sphere::new(vek::Vec3::new(-1., -1.5, -12.), 2., red_rubber));
    spheres.push(Sphere::new(vek::Vec3::new(1.5, -0.5, -18.), 3., red_rubber));
    spheres.push(Sphere::new(vek::Vec3::new(7., 5., -18.), 4., ivory));

    render(&spheres);
}
