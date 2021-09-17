#![feature(exact_size_is_empty)]

use bevy::{input::mouse::MouseMotion, prelude::*};

pub mod collide;
pub mod base;
pub mod asteroids;
pub mod rocket;

use asteroids::{add_asteroid, asteroid_changed, Asteroid};
use base::{add_base, Base};
use rocket::{Rocket, MovingWeapon, WeaponMaterials, RocketLaunch, RocketExplode, rocket_explode, rocket_launching_system, rocket_move_update, rocket_fuel_update};


#[derive(PartialEq, Eq, Debug)]
enum TurnPhase {
    Aiming, 
    Firing
}

impl Default for TurnPhase {
    fn default() -> Self {
        TurnPhase::Aiming
    }
}

#[derive(Default)]
struct TurnState {
    phase : TurnPhase,
    active_base : Option<Entity>,
    firing_angle : f32,
    power : f32
}

// Event
struct TurnPhaseChanged {
    new_phase : TurnPhase
}

// Resource

pub struct UIMaterials {
    retical : Handle<ColorMaterial>
}

impl FromWorld for UIMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let retical_texture_handle = asset_server.load("images/missile_target_2.png");
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        UIMaterials {
            retical : materials.add(retical_texture_handle.into())
        }
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut turn_state : ResMut<TurnState>
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
    turn_state.active_base = Some(base_1);
}


fn firing_system(
    key_input: Res<Input<KeyCode>>,
    mut events: EventWriter<RocketLaunch>,
    turn_state : Res<TurnState>
) {
    if key_input.just_pressed(KeyCode::Space) && turn_state.phase == TurnPhase::Aiming {
        // Test launch a rockets
        if let Some(base) = turn_state.active_base {
            events.send(RocketLaunch{angle : 0.0, offset : 0.0, thrust : 100.0, parent : base});
            events.send(RocketLaunch{angle : 1.0, offset : 0.0, thrust : 100.0, parent : base});
            events.send(RocketLaunch{angle : -1.0, offset : 0.0, thrust : 100.0, parent : base});
        }
    }
}

fn aiming_system(
    key_input: Res<Input<KeyCode>>,
    mut turn_state : ResMut<TurnState>,
    time : Res<Time>
) {
    if turn_state.phase == TurnPhase::Aiming {
        if key_input.pressed(KeyCode::A) {
            turn_state.firing_angle += time.delta_seconds() * 1.5;
        }
        if key_input.pressed(KeyCode::D) {
            turn_state.firing_angle -= time.delta_seconds() * 1.5;
        }

        turn_state.firing_angle = turn_state.firing_angle.clamp(
            -std::f32::consts::FRAC_PI_2, 
            std::f32::consts::FRAC_PI_2
        );

        if key_input.pressed(KeyCode::W) {
            turn_state.power += time.delta_seconds() * 60.0;
        }
        if key_input.pressed(KeyCode::S) {
            turn_state.power -= time.delta_seconds() * 60.0;
        }

        turn_state.power = turn_state.power.clamp(
            10.0,
            100.0
        );
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

fn rocket_asteroid_collide_system(
    rocket_query : Query<(Entity, &Rocket, &GlobalTransform)>,
    asteroid_query : Query<(&Asteroid, &GlobalTransform)>,
    mut events: EventWriter<RocketExplode>
) {
    for (entity, rocket, rocket_transform) in rocket_query.iter() {
        for (asteroid, asteroid_transform) in asteroid_query.iter() {
            if rocket.bound(rocket_transform).collide(asteroid.bound(asteroid_transform)) {
                events.send(RocketExplode { rocket : entity})
            }
        }
    }
}

fn turn_phase_update( 
    mut turn_state : ResMut<TurnState>, 
    weapon_query : Query<&MovingWeapon>,
    mut events : EventWriter<TurnPhaseChanged>
) {
    if turn_state.phase == TurnPhase::Firing && weapon_query.iter().is_empty() {
        turn_state.phase = TurnPhase::Aiming;
        events.send(TurnPhaseChanged{new_phase : TurnPhase::Aiming});
    } 
    if turn_state.phase == TurnPhase::Aiming && !weapon_query.iter().is_empty() {
        turn_state.phase = TurnPhase::Firing;
        events.send(TurnPhaseChanged{new_phase : TurnPhase::Firing});
    }
}

fn turn_setup(mut events : EventWriter<TurnPhaseChanged>) {
    events.send(TurnPhaseChanged{new_phase : TurnPhase::Aiming});
}


fn aiming_ui_aiming_start(
    mut commands: Commands,
    materials : Res<UIMaterials>,
    turn_state : Res<TurnState>,
    mut events : EventReader<TurnPhaseChanged>
) {
    for event in events.iter() {
        if event.new_phase == TurnPhase::Aiming {

        }
    }
}

fn aiming_ui_aiming_end(
    mut commands: Commands,
    mut events : EventReader<TurnPhaseChanged>
) {
    for event in events.iter() {
        if event.new_phase == TurnPhase::Firing {

        }
    }
}

fn main() {
    App::build().insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
                .add_plugins(DefaultPlugins)
                .add_event::<RocketLaunch>()
                .add_event::<RocketExplode>()
                .add_event::<TurnPhaseChanged>()
                .init_resource::<WeaponMaterials>()
                .init_resource::<UIMaterials>()
                .init_resource::<TurnState>()
                .add_startup_system(setup.system())
                .add_system(asteroid_changed.system())
                .add_system(rocket_launching_system.system())
                .add_system(rocket_move_update.system())
                .add_system(rocket_fuel_update.system())
                .add_system(firing_system.system())
                .add_system(gravity_system.system())
                .add_system(rocket_asteroid_collide_system.system())
                .add_system(rocket_explode.system())
                .add_system(turn_phase_update.system())
                .add_system(aiming_system.system())
                .add_startup_system(turn_setup.system())
                .add_system(aiming_ui_aiming_start.system())
                .add_system(aiming_ui_aiming_end.system())
                .run();
}
