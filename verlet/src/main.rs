use std::time::{self, SystemTime};

use nannou::color::*;
use nannou::event::*;
use nannou::prelude::*;

use utils::vec::Vec2;

#[derive(Clone, Debug)]
pub struct Particle {
    pub pos: Vec2,
    pub pos_last: Vec2,
    pub acc: Vec2,
    pub radius: f32,
    pub color: nannou::color::Rgb8,
}

impl Particle {
    pub fn new(pos: Vec2) -> Self {
        Particle {
            pos: pos.clone(),
            pos_last: pos.clone(),
            acc: Vec2::zero(),
            radius: 20_f32,
            color: nannou::color::STEELBLUE,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let delta = self.pos.clone() - self.pos_last.clone();
        self.pos_last = self.pos.clone();
        self.pos += delta + self.acc.clone() * dt * dt;
        self.acc = Vec2::zero();
    }

    pub fn accelerate(&mut self, acc: Vec2) {
        self.acc += acc;
    }

    pub fn set_velocity(mut self, v: Vec2, dt: f32) {
        self.pos_last = self.pos - (v * dt);
    }

    pub fn add_velocity(mut self, v: Vec2, dt: f32) {
        self.pos_last -= v * dt;
    }

    pub fn velocity(self, dt: f32) -> Vec2 {
        (self.pos - self.pos_last) / dt
    }
}

struct Model {
    particles: Vec<Particle>,
    gravity: Vec2,
    center: Vec2,
    last_push: SystemTime,
    mouse_pressed: bool,
}

impl Model {
    pub fn apply_gravity(&mut self) {
        for m in self.particles.iter_mut() {
            m.accelerate(self.gravity.clone());
        }
    }
    pub fn update(&mut self, dt: f32) {
        for m in self.particles.iter_mut() {
            m.update(dt)
        }
    }

    pub fn apply_constraints(&mut self) {
        let constraint_center = self.center.clone();
        let constraint_radius = 300_f32;

        for m in self.particles.iter_mut() {
            let v = constraint_center.clone() - m.pos.clone();
            let dist = v.len();
            if dist > (constraint_radius - m.radius) {
                let n = v / dist;
                m.pos = constraint_center.clone() - n * (constraint_radius - m.radius);
            }
        }
    }

    pub fn solve_collisions(&mut self) {
        let response_coef = 0.8_f32;
        for i in 0..self.particles.len() {
            let o_1 = &self.particles[i].clone();
            for k in (i + 1)..self.particles.len() {
                let o_2 = self.particles[k].clone();
                let v = o_1.pos.clone() - o_2.pos.clone();
                let dist2 = v.x * v.x + v.y * v.y;
                let min_dist = o_1.radius + o_2.radius + 2_f32;
                if dist2 < min_dist * min_dist {
                    let dist = f32::sqrt(dist2);
                    let n = v / dist;
                    let mass_ratio_1 = o_1.radius / (o_1.radius + o_2.radius);
                    let mass_ratio_2 = o_2.radius / (o_1.radius + o_2.radius);
                    let delta = 0.5_f32 * response_coef * (dist - min_dist);

                    self.particles[i].pos -= n.clone() * (mass_ratio_2 * delta);
                    self.particles[k].pos += n * (mass_ratio_1 * delta);
                }
            }
        }
    }
}

fn main() {
    nannou::app(model)
        .simple_window(view)
        .update(update)
        .event(events)
        .run();
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::rate_fps(60.0));
    Model {
        particles: Vec::new(),
        gravity: Vec2::new(0_f32, -1000_f32),
        center: Vec2::new(0_f32, 0_f32),
        last_push: time::SystemTime::now(),
        mouse_pressed: false,
    }
}

fn events(_app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {
            id: id,
            simple: Some(WindowEvent::MouseMoved(p)),
        } if model.mouse_pressed => {
            model.center.x = p[0];
            model.center.y = p[1];
        }

        Event::WindowEvent {
            id: id,
            simple: Some(WindowEvent::MousePressed(MouseButton::Left)),
        } => model.mouse_pressed = true,

        Event::WindowEvent {
            id: id,
            simple: Some(WindowEvent::MouseReleased(MouseButton::Left)),
        } => model.mouse_pressed = false,
        _ => {}
    }
}

fn update(app: &App, model: &mut Model, upd: Update) {
    let now = time::SystemTime::now();

    let elapsed = now.duration_since(model.last_push).unwrap().as_millis();
    if elapsed > 500 && model.particles.len() < 20 {
        model.particles.push(Particle::new(Vec2::new(
            model.center.x + 100_f32,
            model.center.y + 200_f32,
        )));
        model.last_push = now;
    }

    let dt = upd.since_last.as_secs_f32();

    model.apply_gravity();
    model.solve_collisions();
    model.apply_constraints();
    model.update(dt);
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(BLACK);

    draw.ellipse()
        .x(model.center.x)
        .y(model.center.y)
        .color(WHITE)
        .radius(300_f32);

    for m in model.particles.iter() {
        draw.ellipse()
            .color(m.color)
            .x(m.pos.x)
            .y(m.pos.y)
            .radius(m.radius);
    }
    draw.to_frame(app, &frame).unwrap();
}
