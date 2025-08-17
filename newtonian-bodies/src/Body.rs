use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body {
    pub name: String,
    pub mass: f64,
    pub position: Vector,
    pub velocity: Vector,

    #[serde(default = "Vector::default")]
    pub acceleration: Vector,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector {
    pub fn default() -> Self {
        Vector {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}
