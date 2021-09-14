use bevy::prelude::*;

struct Asteroid;

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    // Asteroids
    let asteroid_texture_handle = asset_server.load("images/pallas_asteroid_alpha.png");
    let asteroid_material = materials.add(asteroid_texture_handle.into());
    add_asteroid(&mut commands, 0.0, -215.0, asteroid_material.clone());
    add_asteroid(&mut commands, -60.0, 0.0, asteroid_material.clone());
    add_asteroid(&mut commands, 60.0, 0.0, asteroid_material.clone());
}

fn add_asteroid(commands: &mut Commands, x : f32, y : f32, texture : Handle<ColorMaterial>) {
    commands.spawn_bundle(SpriteBundle {
        material: texture,
        transform: Transform::from_xyz(x, y, 0.0),
        sprite: Sprite::new(Vec2::new(60.0, 60.0)),
        ..Default::default()
    }).insert(Asteroid);
}


fn main() {
    App::build().add_plugins(DefaultPlugins)
                .add_startup_system(setup.system())
                .run();
}
