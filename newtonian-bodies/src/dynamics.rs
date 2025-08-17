use super::Body;
use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};

const G: f64 = 6.67430e-11; // gravitational constant

pub fn simulate(
    bodies: &mut Vec<Body>,
    total_time: f64,
    dt: f64,
    record_interval: u64,
    writer: &mut impl SequentialWriter,
) -> Result<(), Box<dyn Error>> {
    let steps = (total_time as f64 / dt).ceil() as usize;
    let record_steps = (record_interval as f64 / dt).ceil() as usize;

    // 1. Setup the progress bar
    let pb = ProgressBar::new(record_steps as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
        .unwrap()
        .progress_chars("=>-"));

    let total_intervals = (steps as f64 / record_steps as f64).ceil() as u32;
    
    for step in 0..steps {
        // 2. Update the message at the start of each interval
        if step % record_steps == 0 {
            let current_interval = (step / record_steps) + 1;
            pb.set_message(format!("Interval {}/{}", current_interval, total_intervals));
            writer.add(step as u64, bodies)?;
        }

        update_acceleration(bodies);
        update_velocity(bodies, dt);
        update_position(bodies, dt);

        // 3. Set the position. The modulo operator makes it "restart".
        pb.set_position((step % record_steps) as u64 + 1);
    }

    // 4. Finish the progress bar
    pb.finish_with_message("Simulation complete!");

    Ok(())
}

pub trait SequentialWriter {
    fn add(&mut self, time: u64, bodies: &[Body]) -> Result<(), Box<dyn Error>>;
}

pub fn update_acceleration(bodies: &mut Vec<Body>) {
    let bodies_clone = bodies.clone();

    for body in bodies.iter_mut() {
        let mut ax = 0.0;
        let mut ay = 0.0;
        let mut az = 0.0;

        for other in bodies_clone.iter() {
            if body.name == other.name {
                continue;
            }

            let dx = other.position.x - body.position.x;
            let dy = other.position.y - body.position.y;
            let dz = other.position.z - body.position.z;

            let r = (dx * dx + dy * dy + dz * dz).sqrt();
            let f = G * body.mass * other.mass / (r * r);

            ax += f * dx / (r * body.mass);
            ay += f * dy / (r * body.mass);
            az += f * dz / (r * body.mass);
        }

        body.acceleration.x = ax;
        body.acceleration.y = ay;
        body.acceleration.z = az;
    }
}

pub fn update_velocity(bodies: &mut [Body], dt: f64) {
    for body in bodies.iter_mut() {
        body.velocity.x += body.acceleration.x * dt;
        body.velocity.y += body.acceleration.y * dt;
        body.velocity.z += body.acceleration.z * dt;
    }
}

pub fn update_position(bodies: &mut [Body], dt: f64) {
    for body in bodies.iter_mut() {
        body.position.x += body.velocity.x * dt;
        body.position.y += body.velocity.y * dt;
        body.position.z += body.velocity.z * dt;
    }
}