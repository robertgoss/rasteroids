use bevy::prelude::*;

struct AsteroidDrawable;
struct Asteroid{
    max_radius : f32,
    radius : f32
}

struct Base {
    angle : f32,
    offset : f32
}

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
    let ast_1 = add_asteroid(&mut commands, 0.0, -215.0, asteroid_material.clone());
    add_asteroid(&mut commands, -60.0, 0.0, asteroid_material.clone());
    add_asteroid(&mut commands, 60.0, 0.0, asteroid_material.clone());
    // Bases
    let base_texture_handle = asset_server.load("images/base.png");
    let base_material = materials.add(base_texture_handle.into());
    add_base(&mut commands, 0.0, base_material.clone(), ast_1);
    add_base(&mut commands, 1.0, base_material.clone(), ast_1);
    add_base(&mut commands, 2.0, base_material.clone(), ast_1);
    // Set asteroid sizes
    
}

fn add_asteroid(commands: &mut Commands, x : f32, y : f32, texture : Handle<ColorMaterial>) -> Entity{
    let max_radius = 100.0;
    commands.spawn().insert(Asteroid{
        max_radius : 100.0,
        radius : 50.0
    }).insert(Transform::from_xyz(x, y, 0.0)
    ).insert(GlobalTransform::from_xyz(x, y, 0.0)
    ).with_children(
        |parent| {
            parent.spawn_bundle(SpriteBundle {
                material: texture,
                transform: Transform::identity(),
                sprite: Sprite::new(Vec2::new(2.0*max_radius, 2.0*max_radius)),
                ..Default::default()
            }).insert(AsteroidDrawable);
        }
    ).id()
}

fn add_base(commands: &mut Commands, angle : f32, texture : Handle<ColorMaterial>, asteroid : Entity) {
    commands.spawn_bundle(SpriteBundle {
        material: texture,
        transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).insert(Base{
        angle : angle,
        offset : -8.5
    }).insert(Parent(asteroid));
}

// If an asteroid's radius changes we want to update it's sprite and reposition bases on the surface
// so they stay on the surface
fn asteroid_changed(
    query: Query<(&Asteroid, &Children), 
    Changed<Asteroid>>,
    mut transform_query: Query<&mut Transform>,
    bases_query: Query<&Base>,
    asteroid_drawable_query: Query<&AsteroidDrawable>
) {
    for (asteroid, children) in query.iter() {
        // Reposition any bases and update the drawable
        for child in children.iter() {
            // If this is a base reposition it
            if let Ok(base) = bases_query.get(*child) {
                if let Ok(mut transform) = transform_query.get_mut(*child) {
                    let angle = base.angle;
                    let radius = asteroid.radius - base.offset;
                    transform.translation = Vec3::new(-radius * angle.sin(), radius * angle.cos(), 0.0);
                }
            } 
            // Asteroid drawable children need to be scales to new size
            if asteroid_drawable_query.get(*child).is_ok() {
                if let Ok(mut transform) = transform_query.get_mut(*child) {
                    let scale = asteroid.radius / asteroid.max_radius;
                    transform.scale = Vec3::new(scale, scale, scale);
                }
            }
        }
    }
}


fn main() {
    App::build().add_plugins(DefaultPlugins)
                .add_startup_system(setup.system())
                .add_system(asteroid_changed.system())
                .run();
}
