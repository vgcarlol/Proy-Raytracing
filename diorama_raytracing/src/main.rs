mod framebuffer;
mod ray_intersect;
mod color;
mod camera;
mod light;
mod material;
mod cube;
mod texture;

use minifb::{Window, WindowOptions, Key};
use nalgebra_glm::{Vec3, vec3};
use std::time::Duration;
use std::rc::Rc;

use crate::ray_intersect::RayIntersect;
use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::camera::Camera;
use crate::light::Light;
use crate::cube::Cube;
use crate::texture::Texture;

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "House Diorama Scene",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    // Implementar un Skybox
    let skybox_color_day = Color::new(135, 206, 235); // Color cielo claro
    let skybox_color_night = Color::new(0, 0, 40); // Color cielo noche
    let mut skybox_color = skybox_color_day.clone();

    // Cargar texturas
    let brick_texture = Rc::new(Texture::new("assets/brick_texture.jpg"));
    let wood_texture = Rc::new(Texture::new("assets/wood_texture.jpg"));
    let emissive_texture = Rc::new(Texture::new("assets/emissive_texture.jpg")); // Emissive texture

    // Crear objetos de la escena
    let floor = Cube::new(
        Vec3::new(-10.0, -1.0, -10.0),
        Vec3::new(10.0, 0.0, 10.0),
        wood_texture.clone(),
        wood_texture.clone(),
        wood_texture.clone(),
    );

    // Crear las paredes de ladrillo
    let wall1 = Cube::new(
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(10.0, 5.0, -9.0),
        brick_texture.clone(),
        brick_texture.clone(),
        brick_texture.clone(),
    );

    let wall2 = Cube::new(
        Vec3::new(-10.0, 0.0, 9.0),
        Vec3::new(10.0, 5.0, 10.0),
        brick_texture.clone(),
        brick_texture.clone(),
        brick_texture.clone(),
    );

    let wall3 = Cube::new(
        Vec3::new(-10.0, 0.0, -10.0),
        Vec3::new(-9.0, 5.0, 10.0),
        brick_texture.clone(),
        brick_texture.clone(),
        brick_texture.clone(),
    );

    let wall4 = Cube::new(
        Vec3::new(9.0, 0.0, -10.0),
        Vec3::new(10.0, 5.0, 10.0),
        brick_texture.clone(),
        brick_texture.clone(),
        brick_texture.clone(),
    );

    // Crear un cubo emisivo en el centro
    let emissive_cube = Cube::new(
        Vec3::new(-2.0, 0.0, -2.0),
        Vec3::new(2.0, 3.0, 2.0),
        emissive_texture.clone(),
        emissive_texture.clone(),
        emissive_texture.clone(),
    );

    let mut camera = Camera::new(
        Vec3::new(0.0, 2.0, 15.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );

    // Luces iniciales
    let light1 = Light::new(
        Vec3::new(0.0, 5.0, 5.0),
        [255, 255, 255],
        1.0,
    );

    // Agregar una luz exterior
    let light2 = Light::new(
        Vec3::new(15.0, 10.0, 15.0),  // Posición exterior
        [255, 200, 150],              // Color cálido para la luz exterior
        1.5,                          // Intensidad de la luz
    );

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        Box::new(floor),
        Box::new(wall1),
        Box::new(wall2),
        Box::new(wall3),
        Box::new(wall4),
        Box::new(emissive_cube),  // Emissive cube
    ];

    // Ciclo de renderizado y control del ciclo día/noche
    let mut time_of_day = 0.0;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Actualizar la cámara
        if window.is_key_down(Key::Left) {
            camera.orbit(0.1, 0.0);
        }

        if window.is_key_down(Key::Right) {
            camera.orbit(-0.1, 0.0);
        }

        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, 0.1);
        }

        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, -0.1);
        }

        if window.is_key_down(Key::W) {
            camera.zoom(0.1);  // Acercar cámara
        }

        if window.is_key_down(Key::S) {
            camera.zoom(-0.1);  // Alejar cámara
        }

        // Control del ciclo día/noche
        time_of_day += 0.01;
        if time_of_day > 1.0 {
            time_of_day = 0.0;
        }
        skybox_color = if time_of_day < 0.5 { skybox_color_day.clone() } else { skybox_color_night.clone() };

        // Clonar las luces antes de pasarlas a render_scene
        render_scene(&mut framebuffer, &objects, &camera, &[light1.clone(), light2.clone()], &skybox_color);

        // Actualizar ventana con el framebuffer
        window.update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height).unwrap();

        std::thread::sleep(frame_delay);
    }
}

fn render_scene(
    framebuffer: &mut Framebuffer,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    lights: &[Light],
    skybox_color: &Color,
) {
    let width = framebuffer.width as f32;
    let height = framebuffer.height as f32;
    let aspect_ratio = width / height;
    let fov = std::f32::consts::PI / 3.0;  // Cambié "." a "::" aquí
    let perspective_scale = (fov * 0.5).tan();

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            let screen_x = (2.0 * x as f32 / width) - 1.0;
            let screen_y = 1.0 - (2.0 * y as f32 / height);
            let ray_direction = vec3(
                screen_x * aspect_ratio * perspective_scale,
                screen_y * perspective_scale,
                -1.0,
            )
            .normalize();

            let rotated_ray_direction = camera.base_change(&ray_direction);
            let color = cast_ray(&camera.eye, &rotated_ray_direction, objects, lights, 0, skybox_color);
            framebuffer.point(x, y, color.to_hex());
        }
    }
}

fn cast_ray(
    ray_origin: &Vec3,
    ray_direction: &Vec3,
    objects: &[Box<dyn RayIntersect>],
    lights: &[Light],
    _depth: u32,
    skybox_color: &Color,
) -> Color {
    let mut closest_intersection = None;
    let mut zbuffer = f32::INFINITY;

    for object in objects {
        let intersection = object.ray_intersect(ray_origin, ray_direction);
        if intersection.is_intersecting && intersection.distance < zbuffer {
            zbuffer = intersection.distance;
            closest_intersection = Some(intersection);
        }
    }

    if let Some(intersection) = closest_intersection {
        let mut final_color = Color::new(0, 0, 0);

        for light in lights {
            let light_dir = (light.position - intersection.point).normalize();
            let intensity = light_dir.dot(&intersection.normal).max(0.0);

            // Obtener el color de la textura usando las coordenadas UV
            let texture_color = intersection.material.get_texture_color(
                intersection.u.unwrap_or(0.0),
                intersection.v.unwrap_or(0.0),
            );

            let base_color = Color::new(texture_color[0], texture_color[1], texture_color[2]);

            // Calcular el color final con iluminación
            final_color = final_color + (base_color * intensity);
        }

        return final_color;
    }

    // Si no hay intersección, devolver el color del skybox
    skybox_color.clone()
}
