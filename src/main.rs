#![feature(exact_size_is_empty)]

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

struct FiringBase;

struct Weapon;

struct Rocket {
    thrust : Vec2,
    fuel : f32
}

// Events
struct RocketLaunch {
    angle : f32,
    offset : f32,
    thrust : f32,
    parent : Entity
}

// Resourses 
#[derive(Default)]
struct WeaponMaterials {
    rocket : Handle<ColorMaterial>
}

#[derive(PartialEq, Eq)]
enum TurnState {
    Aiming, 
    Firing
}

impl Default for TurnState {
    fn default() -> Self {
        TurnState::Aiming
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut weapon_materials : ResMut<WeaponMaterials>
) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    // Background sprite
    let star_map_handle = asset_server.load("images/starfield.png");
    commands.spawn_bundle(SpriteBundle {
        material: materials.add(star_map_handle.into()),
        transform: Transform::from_scale(Vec3::new(1.5, 1.5, 1.5)),
        ..Default::default()
    });
    // Asteroids
    let asteroid_texture_handle = asset_server.load("images/pallas_asteroid_alpha.png");
    let asteroid_material = materials.add(asteroid_texture_handle.into());
    let ast_1 = add_asteroid(&mut commands, 0.0, -215.0, asteroid_material.clone());
    let ast_2 = add_asteroid(&mut commands, -60.0, 0.0, asteroid_material.clone());
    let ast_3 = add_asteroid(&mut commands, 60.0, 0.0, asteroid_material.clone());
    // Bases
    let base_texture_handle = asset_server.load("images/base.png");
    let base_material = materials.add(base_texture_handle.into());
    let base_1 = add_base(&mut commands, 0.0, base_material.clone(), ast_1);
    add_base(&mut commands, 1.0, base_material.clone(), ast_1);
    add_base(&mut commands, 2.0, base_material.clone(), ast_2);
    add_base(&mut commands, 3.0, base_material.clone(), ast_3);
    commands.entity(base_1).insert(FiringBase);
    // Load weapon textures
    let rocket_texture_handle = asset_server.load("images/rocket.png");
    weapon_materials.rocket = materials.add(rocket_texture_handle.into());
}

fn add_asteroid(commands: &mut Commands, x : f32, y : f32, texture : Handle<ColorMaterial>) -> Entity {
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

fn add_base(commands: &mut Commands, angle : f32, texture : Handle<ColorMaterial>, asteroid : Entity) -> Entity {
    commands.spawn_bundle(SpriteBundle {
        material: texture,
        transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).insert(Base{
        angle : angle,
        offset : -8.5
    }).insert(Parent(asteroid)
    ).id()
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

fn rocket_launching_system(
    mut events: EventReader<RocketLaunch>,
    mut commands: Commands,
    transform_query: Query<&GlobalTransform>,
    materials : Res<WeaponMaterials>
) {
    for launch_event in events.iter() {
        if let Ok(parent_transform) = transform_query.get(launch_event.parent) { 
            let rocket_rotation = parent_transform.rotation * Quat::from_rotation_z(launch_event.angle);
            let direction = rocket_rotation * Vec3::new(0.0, 1.0 ,0.0);
            let offset = direction * launch_event.offset;
            let thrust = Vec2::new(direction.x, direction.y) * launch_event.thrust;
            commands.spawn_bundle(SpriteBundle {
                material: materials.rocket.clone(),
                transform: Transform { 
                    translation : parent_transform.translation + offset,
                    rotation : rocket_rotation,
                    scale : Vec3::new(1.0,1.0,1.0)
                },
                sprite: Sprite::new(Vec2::new(12.0, 36.0)),
                ..Default::default()
            }).insert(
                Rocket{ thrust : thrust, fuel : 6.0 }
            ).insert(Weapon);
        }
    }
}

fn rocket_move_update(
    mut rocket_query: Query<(&Rocket, &mut Transform)>, 
    time: Res<Time>
) {
    for (rocket, mut transform) in rocket_query.iter_mut() {
        let thrust = Vec3::new(rocket.thrust.x, rocket.thrust.y, 0.0);
        if thrust.length() > 1.0 {
            let goal_rotation = Quat::from_rotation_arc(Vec3::new(0.0,1.0,0.0), thrust.normalize());
            transform.rotation = transform.rotation.lerp(goal_rotation, 0.3);
            transform.translation += thrust * time.delta_seconds();
        }
    }
}

fn rocket_fuel_update(
    mut rocket_query: Query<(Entity, &mut Rocket)>, 
    mut commands: Commands,
    time: Res<Time>
) {
    for (entity, mut rocket) in rocket_query.iter_mut() {
        rocket.fuel -= time.delta_seconds();
        if rocket.fuel < 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn firing_system(
    mouse_button_input: Res<Input<MouseButton>>,
    base_query: Query<Entity, With<FiringBase>>, 
    mut events: EventWriter<RocketLaunch>,
    mut turn_state : ResMut<TurnState>
) {
    if mouse_button_input.just_pressed(MouseButton::Left) && *turn_state == TurnState::Aiming {
        // Test launch a rockets
        for base in base_query.iter() {
            events.send(RocketLaunch{angle : 0.0, offset : 0.0, thrust : 100.0, parent : base});
            events.send(RocketLaunch{angle : 1.0, offset : 0.0, thrust : 100.0, parent : base});
            events.send(RocketLaunch{angle : -1.0, offset : 0.0, thrust : 100.0, parent : base});
        }
        *turn_state = TurnState::Firing;
    }
}

fn gravity_system(
    mut rocket_query : Query<(&mut Rocket, &GlobalTransform)>,
    asteroid_query : Query<(&Asteroid, &GlobalTransform)>
) {
    for (mut rocket, rocket_transform) in rocket_query.iter_mut() {
        for (asteroid, asteroid_transform) in asteroid_query.iter() {
            let delta3 = rocket_transform.translation - asteroid_transform.translation;
            let delta = Vec2::new(delta3.x, delta3.y);
            if delta.length() > 1.0 {
                let mass = 3.0 * asteroid.radius * asteroid.radius;
                let dist_sq = delta.length_squared();
                rocket.thrust -= (mass / dist_sq) * delta.normalize();
            }
        }
    }
}

fn turn_update(
    mut turn_state : ResMut<TurnState>, 
    weapon_query : Query<&Weapon>
) {
    if weapon_query.iter().is_empty() {
        *turn_state = TurnState::Aiming;
    } else {
        *turn_state = TurnState::Firing;
    }
}

fn main() {
    App::build().insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
                .add_plugins(DefaultPlugins)
                .add_event::<RocketLaunch>()
                .init_resource::<WeaponMaterials>()
                .init_resource::<TurnState>()
                .add_startup_system(setup.system())
                .add_system(asteroid_changed.system())
                .add_system(rocket_launching_system.system())
                .add_system(rocket_move_update.system())
                .add_system(rocket_fuel_update.system())
                .add_system(firing_system.system())
                .add_system(gravity_system.system())
                .add_system(turn_update.system())
                .run();
}
