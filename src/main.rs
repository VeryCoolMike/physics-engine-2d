extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{FPoint, Point, Rect};
use sdl2::render::Canvas;
use sdl2::sys::{ResizeRedirectMask, SDL_GetMouseState};
use sdl2::video::Window;

use core::f64;
use std::num::FpCategory;
use std::time::{Duration, SystemTime};
use rand::prelude::*;

// For now, we assume that all objects are squares! (Because they are!)
#[derive(Clone, Copy)]
struct Object {
    position: FPoint,
    rotation: f32,
    size: FPoint,
    velocity: FPoint,
    anchored: bool,
}

#[derive(Clone, Copy)]
struct Line {
    origin: FPoint,
    rotation: f32
}

impl Object {
    fn new() -> Self {
        Self {
            position: FPoint::new(0.0, 0.0),
            rotation: 0.0,
            size: FPoint::new(50.0, 50.0),
            velocity: FPoint::new(0.0, 0.0),
            anchored: false
        }
    }

    fn draw(&self, canvas: &mut Canvas<Window>) {
        let corners = self.get_corners();

        let _ = canvas.draw_fline(
            corners[0],
            corners[1]
        );

        let _ = canvas.draw_fline(
            corners[1],
            corners[2]
        );

        let _ = canvas.draw_fline(
            corners[2],
            corners[3]
        );

        let _ = canvas.draw_fline(
            corners[3],
            corners[0]
        );

        let _ = canvas.draw_fline(
            corners[0],
            corners[2]
        );
    }

    fn get_corners(&self) -> [FPoint; 4] {
        let rotation_sin = self.rotation.to_radians().sin();
        let rotation_cos = self.rotation.to_radians().cos();

        let point_0 = rotate_point_around(
            &FPoint::new(self.position.x - self.size.x / 2.0, self.position.y - self.size.y / 2.0),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        let point_1 = rotate_point_around(
            &FPoint::new(self.position.x + self.size.x / 2.0, self.position.y - self.size.y / 2.0),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        let point_2 = rotate_point_around(
            &FPoint::new(self.position.x + self.size.x / 2.0, self.position.y + self.size.y / 2.0),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        let point_3 = rotate_point_around(
            &FPoint::new(self.position.x - self.size.x / 2.0, self.position.y + self.size.y / 2.0),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        return [point_0, point_1, point_2, point_3];
    }

    fn get_edges(&self) -> [FPoint; 4] {
        let rotation_sin = self.rotation.to_radians().sin();
        let rotation_cos = self.rotation.to_radians().cos();

        let edge_0 = rotate_point_around(
            &FPoint::new(self.position.x, self.position.y - self.size.y / 2.0),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        let edge_1 = rotate_point_around(
            &FPoint::new(self.position.x + self.size.x / 2.0, self.position.y),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        let edge_2 = rotate_point_around(
            &FPoint::new(self.position.x, self.position.y + self.size.y / 2.0),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        let edge_3 = rotate_point_around(
            &FPoint::new(self.position.x - self.size.x / 2.0, self.position.y),
            &self.position,
            rotation_sin,
            rotation_cos
        );

        return [edge_0, edge_1, edge_2, edge_3];
    }

    fn get_normals(&self) -> [FPoint; 4] {
        let corners = self.get_corners();

        let edge_vector_0 = corners[1] - corners[0];
        let edge_vector_1 = corners[2] - corners[1];
        let edge_vector_2 = corners[3] - corners[2];
        let edge_vector_3 = corners[0] - corners[3];

        let normal_0 = normalize(FPoint::new(edge_vector_0.y, -edge_vector_0.x));
        let normal_1 = normalize(FPoint::new(edge_vector_1.y, -edge_vector_1.x));
        let normal_2 = normalize(FPoint::new(edge_vector_2.y, -edge_vector_2.x));
        let normal_3 = normalize(FPoint::new(edge_vector_3.y, -edge_vector_3.x));

        return [normal_0, normal_1, normal_2, normal_3]; 
    }
}

fn get_length(point: FPoint) -> f32 {
    return (point.x*point.x + point.y*point.y).sqrt();
}

fn normalize(point: FPoint) -> FPoint {
    let point_length = get_length(point);
    return FPoint::new(point.x / point_length, point.y / point_length);
}

fn dot(point_1: FPoint, point_2: FPoint) -> f32 {
    return point_1.x * point_2.x + point_1.y * point_2.y; 
}

fn project(point: FPoint, axis: FPoint) -> f32 {
    return dot(axis, point);
}

fn overlap(min_a: f32, max_a: f32, min_b: f32, max_b: f32) -> bool {
    return !(max_a < min_b || max_b < min_a);
}

fn check_broad_collision(shape_a: &Object, shape_b: &Object) -> bool {
    let distance = f32::sqrt(f32::powf(shape_b.position.x - shape_a.position.x, 2.0) + f32::powf(shape_b.position.y - shape_a.position.y, 2.0));

    let radius_a = 0.5 * f32::sqrt(f32::powi(shape_a.size.x, 2) + f32::powi(shape_a.size.y, 2));
    let radius_b = 0.5 * f32::sqrt(f32::powi(shape_b.size.x, 2) + f32::powi(shape_b.size.y, 2));

    let combination_radius = radius_a + radius_b;

    return distance < combination_radius;
}

fn check_precise_collision(canvas: &mut Canvas<Window>, debug: bool, shape_a: &Object, shape_b: &Object) -> (bool, f32, FPoint) {
    if debug {
        canvas.set_draw_color(Color::RGB(255, 0, 0));
    }
    let edges_a = shape_a.get_edges();
    let normals_a = shape_a.get_normals();
    let corners_a = shape_a.get_corners();

    let edges_b = shape_b.get_edges();
    let normals_b = shape_b.get_normals();
    let corners_b = shape_b.get_corners();

    let mut smallest_overlap = f32::MAX;
    let mut collision_normal = FPoint::new(0.0, 0.0);


    for (edge, normal) in edges_a.iter().zip(normals_a.iter()) {
        if debug {
            canvas.draw_fline(*edge, *edge + *normal * 1000.0);
        }
        let mut min_a = project(corners_a[0], *normal);
        let mut max_a = min_a;

        let mut min_b = project(corners_b[0], *normal);
        let mut max_b = min_b;

        for corner in corners_a {
            let points = project(corner, *normal);
            if points > max_a {
                max_a = points;
            }
            if points < min_a {
                min_a = points;
            }
        }

        for corner in corners_b {
            let points = project(corner, *normal);
            if points > max_b {
                max_b = points;
            }
            if points < min_b {
                min_b = points;
            }
        }

        let overlap_amount = f32::min(max_a, max_b) - f32::max(min_a, min_b);
        if overlap_amount < smallest_overlap {
            smallest_overlap = overlap_amount;
            collision_normal = *normal;
            let direction = shape_b.position - shape_a.position;
            if dot(direction, collision_normal) < 0.0 {
                collision_normal = -collision_normal;
            }
        }

        if !overlap(min_a, max_a, min_b, max_b) {
            return (false, smallest_overlap, collision_normal);
        }
    }

    if debug {
        canvas.set_draw_color(Color::RGB(0, 255, 0));
    }

    for (edge, normal) in edges_b.iter().zip(normals_b.iter()) {
        if debug {
            canvas.draw_fline(*edge, *edge + *normal * 1000.0);
        }
        let mut min_a = project(corners_a[0], *normal);
        let mut max_a = min_a;

        let mut min_b = project(corners_b[0], *normal);
        let mut max_b = min_b;

        for corner in corners_a {
            let points = project(corner, *normal);
            if points > max_a {
                max_a = points;
            }
            if points < min_a {
                min_a = points;
            }
        }

        for corner in corners_b {
            let points = project(corner, *normal);
            if points > max_b {
                max_b = points;
            }
            if points < min_b {
                min_b = points;
            }
        }

        let overlap_amount = f32::min(max_a, max_b) - f32::max(min_a, min_b);
        if overlap_amount < smallest_overlap {
            smallest_overlap = overlap_amount;
            collision_normal = *normal;
            let direction = shape_b.position - shape_a.position;
            if dot(direction, collision_normal) < 0.0 {
                collision_normal = -collision_normal;
            }
        }

        if !overlap(min_a, max_a, min_b, max_b) {
            return (false, smallest_overlap, collision_normal);
        }
    }

    if debug {
        canvas.set_draw_color(Color::RGB(255, 255, 255));
    }

    return (true, smallest_overlap, collision_normal);
}

fn resolve_collision(shape_a: &mut Object, shape_b: &mut Object, overlap: f32, normal: FPoint) {
    if !shape_a.anchored {
        shape_a.position -= normal * overlap / 2.0;
    }
    if !shape_b.anchored {
        shape_b.position += normal * overlap / 2.0;
    }
}

fn resolve_velocities(shape_a: &mut Object, shape_b: &mut Object, normal: FPoint) {
    let mass_a = shape_a.size.x * shape_a.size.y;
    let mass_b = shape_b.size.x * shape_b.size.y;
    let relative_velocity = shape_b.velocity - shape_a.velocity;
    let relative_velocity_along_normal = dot(relative_velocity, normal); 

    let restitution = 0.5;
    let impulse_numerator = -(1.0 + restitution) * relative_velocity_along_normal;
    let impulse_denominator = 1.0 / mass_a + 1.0 / mass_b; 

    let impulse_scalar = impulse_numerator / impulse_denominator;
    let impulse = normal * impulse_scalar;

    if !shape_a.anchored {
        shape_a.velocity -= impulse / mass_a;
    }
    if !shape_b.anchored {
        shape_b.velocity += impulse / mass_b;
    }
}

fn rotate_point_around(point: &FPoint, orbit: &FPoint, rotation_sin: f32, rotation_cos: f32) -> FPoint {
    let point_rotated = FPoint::new(
        (point.x - orbit.x) * rotation_cos - (point.y - orbit.y) * rotation_sin,
        (point.x - orbit.x) * rotation_sin + (point.y - orbit.y) * rotation_cos
    );

    return FPoint::new(point_rotated.x + orbit.x, point_rotated.y + orbit.y);
}

// DANGER!
// This actually returns a vector, not a point because of rounding
fn rotate_point(point: &FPoint, rotation: f32) -> FPoint {
    let new_rotation = rotation.to_radians();
    let new_x = point.x as f32 * f32::cos(new_rotation) - point.y as f32 * f32::sin(new_rotation);
    let new_y = point.x as f32 * f32::sin(new_rotation) + point.y as f32 * f32::cos(new_rotation);

    return FPoint::new(new_x, new_y); // Vector2 my beloved :heart:
}

fn main() {
    let mut rng = rand::rng();

    const FPS: u32 = 240;

    let mut delta_time: f32 = 1.0/FPS as f32;

    let mut object_list: Vec<Object> = vec![];

    let mut active = true;

    /* 
    let mut example = Object::new();
    example.position = FPoint::new(300.0, 100.0);
    example.anchored = false; 
    example.rotation = 0.0;
    example.size = FPoint::new(50.0, 100.0);

    let mut obstacle = Object::new();
    obstacle.position = FPoint::new(300.0, 500.0);
    obstacle.anchored = true;
    obstacle.size = FPoint::new(200.0, 150.0);
    obstacle.rotation = 45.0;
    */

    let mut floor = Object::new();
    floor.position = FPoint::new(400.0, 700.0);
    floor.anchored = true;
    floor.size = FPoint::new(100000.0, 50.0);

    let mut funny = Object::new();
    funny.position = FPoint::new(0.0, 400.0);
    funny.size = FPoint::new(150.0, 150.0);
    funny.velocity = FPoint::new(850.0, -100.0);
    funny.rotation = 45.0;

    let mut funny2 = Object::new();
    funny2.position = FPoint::new(-5000.0, 600.0);
    funny2.size = FPoint::new(150.0, 150.0);
    funny2.velocity = FPoint::new(3050.0, -100.0);

    let mut funny3 = Object::new();
    funny3.position = FPoint::new(550.0, -10000.0);
    funny3.size = FPoint::new(150.0, 150.0);
    funny3.velocity = FPoint::new(0.0, 10000.0);


    /*
    object_list.push(example);
    object_list.push(obstacle);
    */

    for i in 0..10 {
        for j in 0..3 {
            let mut test_box = Object::new();
            test_box.position = FPoint::new(600.0 - j as f32 * 50.0, 600.0 - i as f32 * 50.0);
            test_box.anchored = false;
            test_box.size = FPoint::new(50.0, 50.0);
            object_list.push(test_box);
        }
    }
    object_list.push(floor);
    object_list.push(funny);
    object_list.push(funny2);
    object_list.push(funny3);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("rust-sdl2 demo", 800, 800)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let frame_begin = SystemTime::now();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        let mouse_state = event_pump.mouse_state();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    active = match active {
                        true => false,
                        false => true
                    };
                },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    let mut test = Object::new();
                    test.position = FPoint::new(
                        mouse_state.x() as f32,
                        mouse_state.y() as f32
                    );
                    test.anchored = false; 
                    test.rotation = 0.0;
                    test.size = FPoint::new(50.0, 50.0);
                    test.velocity = FPoint::new(0.0, 0.0);
                    object_list.push(test);
                },
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        
        /*
        for i in 0..1 {
            let mut test = Object::new();
            test.position = FPoint::new(rng.random_range(0.0..800.0), rng.random_range(0.0..800.0));
            test.anchored = false; 
            test.rotation = rng.random_range(-360.0..360.0);
            test.size = FPoint::new(rng.random_range(0.0..100.0), rng.random_range(0.0..100.0));
            test.velocity = FPoint::new(rng.random_range(-2000.0..2000.0), rng.random_range(-2000.0..2000.0));
            object_list.push(test);
        }
        */

        for i in 0..object_list.len() {
            let element = &mut object_list[i];
            if !element.anchored && active {
                element.velocity += FPoint::new(0.0, 120.0) * delta_time;
                element.position += element.velocity * delta_time;
            }
        }

        for i in 0..object_list.len() {
            let element = &mut object_list[i];
            element.draw(&mut canvas);
        }
        
        for _ in 0..5 {
            for i in 0..object_list.len() {
                let (left, right) = object_list.split_at_mut(i + 1);
                let element = &mut left[i];

                for other in right {
                    if check_broad_collision(element, other) {
                        let (collision, overlap, normal) = check_precise_collision(&mut canvas, true, element, other);
                        if collision {
                            resolve_collision(element, other, overlap, normal);
                            resolve_velocities(element, other, normal);
                        }
                    }
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / FPS));

        println!("{}", 1.0/delta_time);
        let frame_time = SystemTime::now().duration_since(frame_begin).expect("time should go forward");
        delta_time = frame_time.as_secs_f32();
    }
}
