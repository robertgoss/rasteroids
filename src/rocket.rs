use bevy::prelude::*;

use super::collide::Box;

// Components

pub struct MovingWeapon;

pub struct Rocket {
    pub thrust : Vec2,
    pub fuel : f32,
    pub size : Vec2
}

impl Rocket {
    pub fn bound(self : &Self, transform : &GlobalTransform) -> Box {
        let centre = Vec2::new(transform.translation.x, transform.translation.y);
        Box { centre : centre, size : self.size, rotation : transform.rotation }
    }
}


// Events
pub struct RocketLaunch {
    pub angle : f32,
    pub offset : f32,
    pub thrust : f32,
    pub parent : Entity
}

pub struct RocketExplode {
    pub rocket : Entity
}

// Resourses 
pub struct WeaponMaterials {
    rocket : Handle<ColorMaterial>
}

impl FromWorld for WeaponMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let rocket_texture_handle = asset_server.load("images/rocket.png");
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        WeaponMaterials {
            rocket : materials.add(rocket_texture_handle.into())
        }
    }
}

// Systems

pub fn rocket_fuel_update(
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

pub fn rocket_launching_system(
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
            let size = Vec2::new(12.0, 36.0);
            commands.spawn_bundle(SpriteBundle {
                material: materials.rocket.clone(),
                transform: Transform { 
                    translation : parent_transform.translation + offset,
                    rotation : rocket_rotation,
                    scale : Vec3::new(1.0,1.0,1.0)
                },
                sprite: Sprite::new(size),
                ..Default::default()
            }).insert(
                Rocket{ thrust : thrust, fuel : 6.0 , size}
            ).insert(MovingWeapon);
        }
    }
}

pub fn rocket_move_update(
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

pub fn rocket_explode(
    mut events: EventReader<RocketExplode>,
    mut commands: Commands,
    rocket_query : Query<&Rocket>
) {
    for event in events.iter() {
        if rocket_query.get(event.rocket).is_ok() { 
            commands.entity(event.rocket).despawn_recursive();
        }
    }
}