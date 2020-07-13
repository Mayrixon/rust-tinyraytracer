use std::fs::*;
use std::io::{prelude::*, BufWriter};
use vek;

struct Light {
    position: vek::Vec3<f64>,
    intensity: f64,
}

impl Light {
    fn new(position: vek::Vec3<f64>, intensity: f64) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

#[derive(Clone, Copy)]
struct Material {
    refractive_index: f64,
    albedo: [f64; 4],
    diffuse_color: vek::Rgb<f64>,
    specular_exponent: f64,
}

impl Material {
    fn new(
        refractive_index: f64,
        albedo: [f64; 4],
        color: vek::Rgb<f64>,
        specular_exponent: f64,
    ) -> Self {
        Self {
            refractive_index,
            albedo,
            diffuse_color: color,
            specular_exponent,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        let mut albedo = [0.; 4];
        albedo[0] = 1.;
        Self {
            refractive_index: 1.,
            albedo,
            diffuse_color: vek::Rgb::black(),
            specular_exponent: 0.,
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
            }
            if *t0 < 0. {
                return false;
            }
            return true;
        }
    }
}

fn reflect(I: vek::Vec3<f64>, N: vek::Vec3<f64>) -> vek::Vec3<f64> {
    return I - 2. * N * I.dot(N);
}

fn refract(I: vek::Vec3<f64>, N: vek::Vec3<f64>, refractive_index: f64) -> vek::Vec3<f64> {
    let mut cosi = -I.dot(N).max(-1.).min(1.);
    // let mut cosi = -I.dot(N);
    let mut etai = 1.0;
    let mut etat = refractive_index;
    let mut n = N;
    if cosi < 0. {
        cosi = -cosi;
        std::mem::swap(&mut etai, &mut etat);
        n = -n;
    }
    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0. {
        return vek::Vec3::zero();
    } else {
        return I * eta + n * (eta * cosi - k.sqrt());
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

fn cast_ray(
    orig: vek::Vec3<f64>,
    dir: vek::Vec3<f64>,
    spheres: &Vec<Sphere>,
    lights: &Vec<Light>,
    depth: &mut usize,
) -> vek::Rgb<f64> {
    let mut point = vek::Vec3::<f64>::default();
    let mut N = vek::Vec3::<f64>::default();
    let mut material = Material::default();

    if *depth > 4 || !scene_intersect(orig, dir, spheres, &mut point, &mut N, &mut material) {
        return vek::Rgb::new(0.2, 0.7, 0.8);
    }
    let reflect_dir = reflect(dir, N).normalized();
    let refract_dir = refract(dir, N, material.refractive_index).normalized();
    let reflect_orig = if reflect_dir.dot(N) < 0. {
        point - N * 1e-3
    } else {
        point + N * 1e-3
    };
    let refract_orig = if refract_dir.dot(N) < 0. {
        point - N * 1e-3
    } else {
        point + N * 1e-3
    };
    let reflect_color = cast_ray(
        reflect_orig,
        reflect_dir,
        spheres,
        lights,
        &mut (*depth + 1),
    );
    let refract_color = cast_ray(
        refract_orig,
        refract_dir,
        spheres,
        lights,
        &mut (*depth + 1),
    );

    let mut diffuse_light_intensity: f64 = 0.;
    let mut specular_light_intensity: f64 = 0.;
    for i in 0..lights.len() {
        let light_dir = (lights[i].position - point).normalized();
        let light_distance = (lights[i].position - point).magnitude();

        let shadow_orig = if light_dir.dot(N) < 0. {
            point - N * 1e-3
        } else {
            point + N * 1e-3
        };
        let mut shadow_pt = vek::Vec3::<f64>::zero();
        let mut shadow_N = vek::Vec3::<f64>::zero();
        let mut tmp_material = Material::default();
        if scene_intersect(
            shadow_orig,
            light_dir,
            &spheres,
            &mut shadow_pt,
            &mut shadow_N,
            &mut tmp_material,
        ) && (shadow_pt - shadow_orig).magnitude() < light_distance
        {
            continue;
        }

        diffuse_light_intensity += lights[i].intensity * light_dir.dot(N).max(0.);
        specular_light_intensity += reflect(light_dir, N)
            .dot(dir)
            .max(0.)
            .powf(material.specular_exponent);
    }
    return material.diffuse_color * diffuse_light_intensity * material.albedo[0]
        + vek::Rgb::white() * specular_light_intensity * material.albedo[1]
        + reflect_color * material.albedo[2]
        + refract_color * material.albedo[3];
}

fn render(spheres: &Vec<Sphere>, lights: &Vec<Light>) {
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
            framebuffer[j][i] = cast_ray(vek::Vec3::zero(), dir, spheres, lights, &mut 0);
        }
    }

    let file = File::create("./target/out.ppm").unwrap();
    let mut buffer = BufWriter::new(file);
    buffer
        .write_fmt(format_args!("P6\n{} {}\n255\n", WIDTH, HEIGHT))
        .unwrap();

    for i in 0..HEIGHT {
        for j in 0..WIDTH {
            let mut c = framebuffer[i][j];
            let max = c[0].max(c[1].max(c[2]));
            if max > 1. {
                c = c * (1. / max);
            }
            for k in 0..3 {
                let a = (255. * c[k].min(1.).max(0.)) as u8;
                buffer.write(&[a]).unwrap();
            }
        }
    }
    buffer.flush().unwrap();
}

fn main() {
    let ivory = Material::new(1.0, [0.6, 0.3, 0.1, 0.0], vek::Rgb::new(0.4, 0.4, 0.3), 50.);
    let glass = Material::new(
        1.5,
        [0.0, 0.5, 0.1, 0.8],
        vek::Rgb::new(0.6, 0.7, 0.8),
        125.,
    );
    let red_rubber = Material::new(1.0, [0.9, 0.1, 0.0, 0.0], vek::Rgb::new(0.3, 0.1, 0.1), 10.);
    let mirror = Material::new(
        1.0,
        [0.0, 10.0, 0.8, 0.0],
        vek::Rgb::new(1.0, 1.0, 1.0),
        1425.,
    );

    let mut spheres = Vec::default();
    spheres.push(Sphere::new(vek::Vec3::new(-3., 0., -16.), 2., ivory));
    spheres.push(Sphere::new(vek::Vec3::new(-1., -1.5, -12.), 2., glass));
    spheres.push(Sphere::new(vek::Vec3::new(1.5, -0.5, -18.), 3., red_rubber));
    spheres.push(Sphere::new(vek::Vec3::new(7., 5., -18.), 4., mirror));

    let mut lights = Vec::default();
    lights.push(Light::new(vek::Vec3::new(-20., 20., 20.), 1.5));
    lights.push(Light::new(vek::Vec3::new(30., 50., -25.), 1.8));
    lights.push(Light::new(vek::Vec3::new(30., 20., 30.), 1.7));

    render(&spheres, &lights);
}
