use bevy::prelude::*;

#[derive(Component)]
struct Object;

#[derive(Component)]
struct Name(String);

#[derive(Component)]
struct Mass(f64);


fn setup(mut commands: Commands) {
    commands.spawn((Object, Name("Sun".to_string()), Mass(1.9891e30))); // Mass of the Sun in kg
}

fn rotate(
    time: Res<Time>,
    mut query: Query<&Name, With<Object>>,
) {
    for name in &query {
        println!("Rotating {}", name.0);
    }
}


pub struct SolarSystemPlugin;

impl Plugin for SolarSystemPlugin {
    fn build(&self, app: &mut App) {
        // add things to your app here
        app.add_systems(Startup, setup);
        app.add_systems(Update, rotate);
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(SolarSystemPlugin)
        .run();
}
