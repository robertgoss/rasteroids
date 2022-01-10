use bevy::prelude::*;

use super::collide::Box;
use super::turn::TurnEnd;
use super::explosion::Explode;
use super::app_state::AppState;

// Components

#[derive(Component)]
pub struct Weapon {
    pub thrust : Vec2,
    pub fuel : f32,
    pub size : Vec2
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WeaponType {
    Rocket
}

impl WeaponType {
    fn size(self : &Self) -> Vec2 {
        match *self {
            WeaponType::Rocket => Vec2::new(12.0, 36.0),
        }
    }
    fn fuel(self : &Self) -> f32 {
        match *self {
            WeaponType::Rocket => 10.0
        }
    }
}

impl Weapon {
    pub fn bound(self : &Self, transform : &GlobalTransform) -> Box {
        let centre = Vec2::new(transform.translation.x, transform.translation.y);
        Box { centre : centre, size : self.size, rotation : transform.rotation }
    }
}


// Events
pub struct Launch {
    pub angle : f32,
    pub offset : f32,
    pub thrust : f32,
    pub parent : Entity,
    pub weapon_type : WeaponType
}

pub struct WeaponExplode {
    pub entity : Entity
}

// Resourses 
pub struct WeaponMaterials {
    rocket : Handle<Image>
}

impl WeaponMaterials {
    pub fn texture(self : &Self, weapon_type : WeaponType) -> Handle<Image> {
        match weapon_type {
            WeaponType::Rocket => self.rocket.clone()
        }
    }
}

impl FromWorld for WeaponMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let rocket_texture_handle = asset_server.load("images/rocket.png");
        WeaponMaterials {
            rocket : rocket_texture_handle
        }
    }
}

// Systems

pub fn weapon_fuel_update(
    mut weapon_query: Query<(Entity, &mut Weapon)>, 
    mut commands: Commands,
    time: Res<Time>,
    mut events_turn : EventWriter<TurnEnd>
) {
    for (entity, mut weapon) in weapon_query.iter_mut() {
        weapon.fuel -= time.delta_seconds();
        if weapon.fuel < 0.0 {
            events_turn.send(TurnEnd);
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn launch_weapon(
    commands: &mut Commands,
    parent_transform : &GlobalTransform,
    materials : &WeaponMaterials,
    launch : &Launch
) -> Entity {
    let rocket_rotation = parent_transform.rotation * Quat::from_rotation_z(launch.angle);
    let direction = rocket_rotation * Vec3::new(0.0, 1.0 ,0.0);
    let offset = direction * launch.offset;
    let thrust = Vec2::new(direction.x, direction.y) * launch.thrust;
    let size = launch.weapon_type.size();
    commands.spawn_bundle(SpriteBundle {
        texture: materials.texture(launch.weapon_type),
        transform: Transform { 
            translation : parent_transform.translation + offset,
            rotation : rocket_rotation,
            scale : Vec3::new(1.0,1.0,1.0)
        },
        sprite: Sprite { custom_size : Some(size), ..Default::default()},
        ..Default::default()
    }).insert(
        Weapon{ 
            thrust : thrust, 
            fuel : launch.weapon_type.fuel(), 
            size : size
        }
    ).id()
}

pub fn launching_system(
    mut events: EventReader<Launch>,
    mut commands: Commands,
    transform_query: Query<&GlobalTransform>,
    materials : Res<WeaponMaterials>
) {
    for launch_event in events.iter() {
        if let Ok(parent_transform) = transform_query.get(launch_event.parent) { 
            launch_weapon(
                &mut commands, 
                parent_transform, 
                &materials, 
                launch_event
            );
        }
    }
}

pub fn weapon_move_update(
    mut weapon_query: Query<(&Weapon, &mut Transform)>, 
    time: Res<Time>
) {
    for (weapon, mut transform) in weapon_query.iter_mut() {
        let thrust = Vec3::new(weapon.thrust.x, weapon.thrust.y, 0.0);
        if thrust.length() > 1.0 {
            let goal_rotation = Quat::from_rotation_arc(Vec3::new(0.0,1.0,0.0), thrust.normalize());
            transform.rotation = transform.rotation.lerp(goal_rotation, 0.3);
            transform.translation += thrust * time.delta_seconds();
        }
    }
}

pub fn weapon_explode(
    mut events: EventReader<WeaponExplode>,
    mut commands: Commands,
    weapon_query : Query<&GlobalTransform>,
    mut events_turn : EventWriter<TurnEnd>,
    mut events_explosion : EventWriter<Explode>
) {
    for event in events.iter() {
        if let Ok(transform) = weapon_query.get(event.entity) { 
            events_turn.send(TurnEnd);
            let pos = transform.translation;
            events_explosion.send(Explode { pos : Vec2::new(pos.x, pos.y), power : 50.0 } );
            commands.entity(event.entity).despawn_recursive();
        }
    }
}

fn teardown_weapons(
    mut commands : Commands,
    weapon_query : Query<Entity, With<Weapon>>
) {
    for weapon in weapon_query.iter() {
        commands.entity(weapon).despawn_recursive();
    }
}

// Plugins
pub struct WeaponPlugin;

impl Plugin for WeaponPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Launch>()
           .add_event::<WeaponExplode>()
           .init_resource::<WeaponMaterials>()
           .add_system_set(
             SystemSet::on_update(AppState::InGame)
               .with_system(launching_system.system())
               .with_system(weapon_move_update.system())
               .with_system(weapon_fuel_update.system())
               .with_system(weapon_explode.system()))
           .add_system_set(
              SystemSet::on_exit(AppState::InGame)
               .with_system(teardown_weapons.system())
            );
    }
}