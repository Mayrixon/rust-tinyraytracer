use std::fs::*;
use std::io::{BufWriter, prelude::*};

use vek::{Rgb, Vec3};

struct Light {
    position: Vec3<f64>,
    intensity: f64,
}

impl Light {
    fn new(position: Vec3<f64>, intensity: f64) -> Self {
        Self {
            position,
            intensity,
        }
    }
}

#[derive(Default)]
struct Ray {
    origin: Vec3<f64>,
    direction: Vec3<f64>,
}

impl Ray {
    fn new(origin: Vec3<f64>, direction: Vec3<f64>) -> Self {
        Self { origin, direction }
    }
}

#[derive(Clone, Copy)]
struct Material {
    refractive_index: f64,
    albedo: [f64; 4],
    diffuse_color: Rgb<f64>,
    specular_exponent: f64,
}

impl Material {
    fn new(
        refractive_index: f64,
        albedo: [f64; 4],
        color: Rgb<f64>,
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
        Self {
            refractive_index: 1.,
            albedo: [1., 0., 0., 0.],
            diffuse_color: Rgb::black(),
            specular_exponent: 0.,
        }
    }
}

trait Intersect {
    type Output;

    fn intersect(&self, ray: &Ray) -> Option<Self::Output>;
}

#[derive(Clone, Copy)]
struct Sphere {
    center: Vec3<f64>,
    radius: f64,
    material: Material,
}

impl Sphere {
    fn new(center: Vec3<f64>, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}
impl Intersect for Sphere {
    type Output = f64;

    fn intersect(&self, ray: &Ray) -> Option<Self::Output> {
        let v_l = self.center - ray.origin;
        let tca = v_l.dot(ray.direction);
        let d2 = v_l.dot(v_l) - tca * tca;
        let radius2 = self.radius * self.radius;

        if d2 > radius2 {
            None
        } else {
            let thc = (radius2 - d2).sqrt();
            let t0 = tca - thc;
            let t1 = tca + thc;

            if t1 < 0. {
                None
            } else {
                Some(if t0 < 0. { t1 } else { t0 })
            }
        }
    }
}

fn reflect(v_in: &Vec3<f64>, v_normal: &Vec3<f64>) -> Vec3<f64> {
    v_in - 2. * *v_normal * v_in.dot(*v_normal)
}

fn refract(v_in: &Vec3<f64>, v_normal: &Vec3<f64>, refractive_index: f64) -> Vec3<f64> {
    let mut cosi = -v_in.dot(*v_normal).clamp(-1., 1.);
    let mut etai = 1.0;
    let mut etat = refractive_index;
    let mut n = *v_normal;
    if cosi < 0. {
        cosi = -cosi;
        std::mem::swap(&mut etai, &mut etat);
        n = -n;
    }
    let eta = etai / etat;
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);
    if k < 0. {
        Vec3::zero()
    } else {
        v_in * eta + n * (eta * cosi - k.sqrt())
    }
}

struct Scene {
    spheres: Vec<Sphere>,
}

impl Scene {
    fn new(spheres: Vec<Sphere>) -> Self {
        Self { spheres }
    }
}
impl Intersect for Scene {
    type Output = (Vec3<f64>, Vec3<f64>, Material);

    fn intersect(&self, ray: &Ray) -> Option<Self::Output> {
        let mut hit = Vec3::default();
        let mut v_normal = Vec3::default();
        let mut material = Material::default();
        let mut spheres_dist = f64::MAX;

        for sphere in &self.spheres {
            let dist_i = sphere.intersect(ray);

            if let Some(dist_i) = dist_i {
                if dist_i < spheres_dist {
                    spheres_dist = dist_i;
                    hit = ray.origin + ray.direction * dist_i;
                    v_normal = (hit - sphere.center).normalized();
                    material = sphere.material;
                }
            }
        }

        let mut checkerboard_dist = f64::MAX;
        if ray.direction.y.abs() > 1e-3 {
            let d = -(ray.origin.y + 4.) / ray.direction.y;
            let pt = ray.origin + ray.direction * d;
            if d > 0. && pt.x.abs() < 10. && pt.z < -10. && pt.z > -30. && d < spheres_dist {
                checkerboard_dist = d;
                hit = pt;
                v_normal = Vec3::new(0., 1., 0.);
                material.diffuse_color =
                    if ((0.5 * hit.x + 1000.) as isize + (0.5 * hit.z) as isize) & 1 == 1 {
                        Rgb::new(0.3, 0.3, 0.3)
                    } else {
                        Rgb::new(0.3, 0.2, 0.1)
                    };
            }
        }

        if spheres_dist.min(checkerboard_dist) < 1000. {
            Some((hit, v_normal, material))
        } else {
            None
        }
    }
}

fn offset_point(point: &Vec3<f64>, normal: &Vec3<f64>, dot_product: f64) -> Vec3<f64> {
    if dot_product < 0.0 {
        *point - *normal * 1e-3
    } else {
        *point + *normal * 1e-3
    }
}

fn cast_ray(ray: &Ray, spheres: &[Sphere], lights: &[Light], depth: usize) -> (Rgb<f64>, usize) {
    let scene = Scene::new(spheres.to_vec());

    if depth > 4 {
        (Rgb::new(0.2, 0.7, 0.8), depth)
    } else if let Some((point, v_normal, material)) = scene.intersect(ray) {
        let reflect_dir = reflect(&ray.direction, &v_normal).normalized();
        let refract_dir =
            refract(&ray.direction, &v_normal, material.refractive_index).normalized();
        let reflect_orig = offset_point(&point, &v_normal, reflect_dir.dot(v_normal));
        let refract_orig = offset_point(&point, &v_normal, refract_dir.dot(v_normal));
        let reflect_ray = Ray::new(reflect_orig, reflect_dir);
        let refract_ray = Ray::new(refract_orig, refract_dir);
        let (reflect_color, _) = cast_ray(&reflect_ray, spheres, lights, depth + 1);
        let (refract_color, _) = cast_ray(&refract_ray, spheres, lights, depth + 1);

        let mut diffuse_light_intensity: f64 = 0.;
        let mut specular_light_intensity: f64 = 0.;
        for light in lights {
            let v_light = light.position - point;
            let light_dir = v_light.normalized();
            let light_distance = v_light.magnitude();

            let shadow_orig = offset_point(&point, &v_normal, light_dir.dot(v_normal));
            if let Some((shadow_pt, _, _)) = scene.intersect(&Ray::new(shadow_orig, light_dir)) {
                if (shadow_pt - shadow_orig).magnitude() < light_distance {
                    continue;
                }
            }

            diffuse_light_intensity += light.intensity * light_dir.dot(v_normal).max(0.);
            specular_light_intensity += reflect(&light_dir, &v_normal)
                .dot(ray.direction)
                .max(0.)
                .powf(material.specular_exponent);
        }

        (
            material.diffuse_color * diffuse_light_intensity * material.albedo[0]
                + Rgb::white() * specular_light_intensity * material.albedo[1]
                + reflect_color * material.albedo[2]
                + refract_color * material.albedo[3],
            depth,
        )
    } else {
        (Rgb::new(0.2, 0.7, 0.8), depth)
    }
}

fn render(spheres: &[Sphere], lights: &[Light]) {
    const WIDTH: usize = 1024;
    const HEIGHT: usize = 768;
    const FOV: usize = std::f64::consts::FRAC_PI_2 as usize;
    let mut framebuffer = vec![vec![Rgb::<f64>::zero(); WIDTH]; HEIGHT];

    let aspect_ratio = WIDTH as f64 / HEIGHT as f64;
    let scale = (FOV as f64 / 2.).tan();

    for (j, row) in framebuffer.iter_mut().enumerate() {
        for (i, pixel) in row.iter_mut().enumerate() {
            let x = (2. * (i as f64 + 0.5) / WIDTH as f64 - 1.) * scale * aspect_ratio;
            let y = -(2. * (j as f64 + 0.5) / HEIGHT as f64 - 1.) * scale;
            let dir = Vec3::new(x, y, -1.).normalized();
            let ray = Ray::new(Vec3::zero(), dir);
            (*pixel, _) = cast_ray(&ray, spheres, lights, 0);
        }
    }

    let file = File::create("./target/out.ppm").unwrap();
    let mut buffer = BufWriter::new(file);
    buffer
        .write_fmt(format_args!("P6\n{} {}\n255\n", WIDTH, HEIGHT))
        .unwrap();

    for row in framebuffer {
        for pixel in row {
            let max = pixel.iter().fold(0.0, |acc: f64, &x| acc.max(x));
            let scaled_pixel = if max > 1. { pixel / max } else { pixel };
            let a = scaled_pixel
                .iter()
                .map(|x| (255. * x.clamp(0., 1.)) as u8)
                .collect::<Rgb<u8>>();
            buffer.write_all(&a).unwrap();
        }
    }
    buffer.flush().unwrap();
}

fn main() {
    let ivory = Material::new(1.0, [0.6, 0.3, 0.1, 0.0], Rgb::new(0.4, 0.4, 0.3), 50.);
    let glass = Material::new(1.5, [0.0, 0.5, 0.1, 0.8], Rgb::new(0.6, 0.7, 0.8), 125.);
    let red_rubber = Material::new(1.0, [0.9, 0.1, 0.0, 0.0], Rgb::new(0.3, 0.1, 0.1), 10.);
    let mirror = Material::new(1.0, [0.0, 10.0, 0.8, 0.0], Rgb::new(1.0, 1.0, 1.0), 1425.);

    let spheres = vec![
        Sphere::new(Vec3::new(-3., 0., -16.), 2., ivory),
        Sphere::new(Vec3::new(-1., -1.5, -12.), 2., glass),
        Sphere::new(Vec3::new(1.5, -0.5, -18.), 3., red_rubber),
        Sphere::new(Vec3::new(7., 5., -18.), 4., mirror),
    ];

    let lights = vec![
        Light::new(Vec3::new(-20., 20., 20.), 1.5),
        Light::new(Vec3::new(30., 50., -25.), 1.8),
        Light::new(Vec3::new(30., 20., 30.), 1.7),
    ];

    render(&spheres, &lights);
}
