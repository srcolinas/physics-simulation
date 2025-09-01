use super::Body;
use std::error::Error;
use indicatif::{ProgressBar, ProgressStyle};

pub fn simulate(
    bodies: &mut Vec<Body>,
    gravity: f64,
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

        update_acceleration(bodies, gravity);
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

fn update_acceleration(bodies: &mut Vec<Body>, gravity: f64) {
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
            let f = gravity * body.mass * other.mass / (r * r);

            ax += f * dx / (r * body.mass);
            ay += f * dy / (r * body.mass);
            az += f * dz / (r * body.mass);
        }

        body.acceleration.x = ax;
        body.acceleration.y = ay;
        body.acceleration.z = az;
    }
}

fn update_velocity(bodies: &mut [Body], dt: f64) {
    for body in bodies.iter_mut() {
        body.velocity.x += body.acceleration.x * dt;
        body.velocity.y += body.acceleration.y * dt;
        body.velocity.z += body.acceleration.z * dt;
    }
}

fn update_position(bodies: &mut [Body], dt: f64) {
    for body in bodies.iter_mut() {
        body.position.x += body.velocity.x * dt;
        body.position.y += body.velocity.y * dt;
        body.position.z += body.velocity.z * dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Vector;
    use std::collections::HashMap;

    // Mock implementation of SequentialWriter for testing
    struct MockWriter {
        records: HashMap<u64, Vec<Body>>,
    }

    impl MockWriter {
        fn new() -> Self {
            MockWriter {
                records: HashMap::new(),
            }
        }

        fn get_records(&self) -> &HashMap<u64, Vec<Body>> {
            &self.records
        }
    }

    impl SequentialWriter for MockWriter {
        fn add(&mut self, time: u64, bodies: &[Body]) -> Result<(), Box<dyn Error>> {
            self.records.insert(time, bodies.to_vec());
            Ok(())
        }
    }

    // Helper function to create test bodies
    fn create_test_bodies() -> Vec<Body> {
        vec![
            Body {
                name: "Earth".to_string(),
                mass: 5.972e24,
                position: Vector { x: 0.0, y: 0.0, z: 0.0 },
                velocity: Vector { x: 0.0, y: 0.0, z: 0.0 },
                acceleration: Vector::null(),
            },
            Body {
                name: "Moon".to_string(),
                mass: 7.342e22,
                position: Vector { x: 384400000.0, y: 0.0, z: 0.0 },
                velocity: Vector { x: 0.0, y: 1022.0, z: 0.0 },
                acceleration: Vector::null(),
            },
        ]
    }

    #[test]
    fn test_simulate_basic_functionality() {
        let mut bodies = create_test_bodies();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        assert!(!writer.get_records().is_empty());
    }

    #[test]
    fn test_simulate_with_zero_time() {
        let mut bodies = create_test_bodies();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 0.0;
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        // With zero time, no steps are taken, so no records are written
        assert!(writer.get_records().is_empty());
    }

    #[test]
    fn test_simulate_with_small_dt() {
        let mut bodies = create_test_bodies();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.001;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        // With small dt (0.001) and record_interval (1), record_steps = 1000
        // So we only record every 1000 steps, resulting in 1 record for 1000 steps
        assert_eq!(writer.get_records().len(), 1);
    }

    #[test]
    fn test_simulate_with_large_record_interval() {
        let mut bodies = create_test_bodies();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.1;
        let record_interval = 10;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        // With large record_interval, should have fewer records
        assert!(writer.get_records().len() <= 2); // Initial + maybe one more
    }

    #[test]
    fn test_simulate_conserves_mass() {
        let mut bodies = create_test_bodies();
        let initial_mass: f64 = bodies.iter().map(|b| b.mass).sum();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        let final_mass: f64 = bodies.iter().map(|b| b.mass).sum();
        assert!((initial_mass - final_mass).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simulate_updates_positions() {
        let mut bodies = create_test_bodies();
        let initial_positions: Vec<Vector> = bodies.iter().map(|b| b.position.clone()).collect();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        
        // Check that positions have changed
        for (i, body) in bodies.iter().enumerate() {
            let initial = &initial_positions[i];
            let final_pos = &body.position;
            
            // At least one component should have changed
            let dx = (initial.x - final_pos.x).abs();
            let dy = (initial.y - final_pos.y).abs();
            let dz = (initial.z - final_pos.z).abs();
            
            assert!(dx > f64::EPSILON || dy > f64::EPSILON || dz > f64::EPSILON);
        }
    }

    #[test]
    fn test_simulate_updates_velocities() {
        let mut bodies = create_test_bodies();
        let initial_velocities: Vec<Vector> = bodies.iter().map(|b| b.velocity.clone()).collect();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        
        // Check that velocities have changed
        for (i, body) in bodies.iter().enumerate() {
            let initial = &initial_velocities[i];
            let final_vel = &body.velocity;
            
            // At least one component should have changed
            let dvx = (initial.x - final_vel.x).abs();
            let dvy = (initial.y - final_vel.y).abs();
            let dvz = (initial.z - final_vel.z).abs();
            
            assert!(dvx > f64::EPSILON || dvy > f64::EPSILON || dvz > f64::EPSILON);
        }
    }

    #[test]
    fn test_simulate_with_single_body() {
        let mut bodies = vec![
            Body {
                name: "Lonely".to_string(),
                mass: 1.0e24,
                position: Vector { x: 0.0, y: 0.0, z: 0.0 },
                velocity: Vector { x: 0.0, y: 0.0, z: 0.0 },
                acceleration: Vector::null(),
            }
        ];
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = 1.0;
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        assert!(result.is_ok());
        // Single body should not have acceleration changes
        assert!((bodies[0].acceleration.x).abs() < f64::EPSILON);
        assert!((bodies[0].acceleration.y).abs() < f64::EPSILON);
        assert!((bodies[0].acceleration.z).abs() < f64::EPSILON);
    }

    #[test]
    fn test_simulate_error_handling() {
        // Test with invalid parameters
        let mut bodies = create_test_bodies();
        let mut writer = MockWriter::new();
        let gravity = 6.67430e-11;
        let total_time = -1.0; // Invalid negative time
        let dt = 0.1;
        let record_interval = 1;

        let result = simulate(&mut bodies, gravity, total_time, dt, record_interval, &mut writer);
        
        // Should handle negative time gracefully (will result in 0 steps)
        assert!(result.is_ok());
    }
}