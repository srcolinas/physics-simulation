#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;

#[derive(Debug, Clone, Copy)]
struct Vector2d {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
struct PhysicsObject {
    mass: f64,
    acceleration: Vector2d,
    velocity: Vector2d,
    position: Vector2d,
}

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 960.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Solar System Simulator",
        options,
        Box::new(|_cc| Ok(Box::<Simulator>::default())),
    )
}

struct Simulator {
    gravity: f64,
    objects: Vec<PhysicsObject>,
    delta_time: u64,
}

impl Default for Simulator {
    fn default() -> Self {
        Self {
            gravity: 6.67430e-11, // Gravitational constant in m^3 kg^-1 s^-2
            delta_time: 16,       // 60 FPS ~ 16 ms per frame
            objects: [
                PhysicsObject {
                    mass: 5.972e24, // Mass of Earth in kg
                    acceleration: Vector2d { x: 0.0, y: 0.0 },
                    velocity: Vector2d { x: 0.0, y: 0.0 },
                    position: Vector2d { x: 640.0, y: 480.0 }, // Center of 1280x960 frame
                },
                PhysicsObject {
                    mass: 7.348e22, // Mass of Moon in kg
                    acceleration: Vector2d { x: 0.0, y: 0.0 },
                    velocity: Vector2d { x: 0.0, y: 0.0 },
                    position: Vector2d { x: 700.0, y: 480.0 }, // Slightly offset from Earth
                },
            ]
            .to_vec(),
        }
    }
}

impl eframe::App for Simulator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                ui.heading(
                    egui::RichText::new("Solar System Simulator").color(egui::Color32::WHITE),
                );
                let painter = ui.painter();
                let radius = 20.0;

                for object in &mut self.objects {
                    let mut total_force = Vector2d { x: 0.0, y: 0.0 };
                    for object2 in self.objects.iter().cloned() {
                        if object as *const _ != &object2 as *const _ {
                            let dx = object2.position.x - object.position.x;
                            let dy = object2.position.y - object.position.y;
                            let distance = (dx * dx + dy * dy).sqrt().max(1.0); // Avoid division by zero
                            let force =
                                (self.gravity * object.mass * object2.mass) / distance.powi(2);
                            let direction_x = dx / distance;
                            let direction_y = dy / distance;
                            total_force.x += direction_x * force;
                            total_force.y += direction_y * force;
                        }
                    }
                    object.acceleration.x = total_force.x / object.mass;
                    object.acceleration.y = total_force.y / object.mass;
                    object.velocity.x += object.acceleration.x * (self.delta_time as f64);
                    object.velocity.y += object.acceleration.y * (self.delta_time as f64);
                    object.position.x += object.velocity.x * (self.delta_time as f64);
                    object.position.y += object.velocity.y * (self.delta_time as f64);
                    painter.circle(
                        egui::Pos2::new(object.position.x as f32, object.position.y as f32),
                        radius,
                        egui::Color32::WHITE,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                }
            });
    }
}
