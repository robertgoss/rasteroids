use bevy::prelude::*;

use std::collections::HashMap;

use super::turn::{TurnPhase, TurnState, TurnStart, TurnFiring};
use super::app_state::AppState;
use super::asteroids::{Asteroid, calculate_gravity};
use super::base::Base;

// Components

pub struct AimingMaterials {
    tracer : Handle<Image>
}

impl FromWorld for AimingMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let tracer_texture_handle = asset_server.load("images/missile_target_2.png");
        AimingMaterials {
            tracer : tracer_texture_handle
        }
    }
}

#[derive(Component)]
pub struct AimingTracer {
    delay : f32
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
            30.0,
            200.0
        );
    }
}


// Systems
fn aiming_ui_aiming_start(
    mut commands: Commands,
    mut events : EventReader<TurnStart>,
    textures : Res<AimingMaterials>
) {
    for _ in events.iter() {
        commands.spawn().insert(Timer::from_seconds(0.15, true));
        let size = Vec2::new(6.0, 6.0);
        for i in 1..6 {
            commands.spawn_bundle(SpriteBundle {
                texture: textures.tracer.clone(),
                sprite: Sprite { custom_size : Some(size), ..Default::default() },
                ..Default::default()
            }).insert(AimingTracer {delay : (i as f32) * 0.3});
        }
    }
}

fn calculate_position(
    asteroid_query : &Query<(&Asteroid, &GlobalTransform)>,
    start : Vec2,
    length : f32,
    initial_thrust : Vec2,
) -> Vec2 {
    let step = 0.02;
    let step_num : usize = (length / step) as usize;
    let mut pos = start.clone();
    let mut thrust = initial_thrust.clone();
    for _ in 0..step_num  {
        thrust += calculate_gravity(asteroid_query, pos, step);
        pos += thrust * step;
    }
    pos
}

fn aiming_ui_update_system(
    turn_state : Res<TurnState>,
    tracer_query : Query<(Entity, &AimingTracer)>,
    mut query : QuerySet<(
        QueryState<&GlobalTransform, With<Base>>,
        QueryState<(Entity, &mut GlobalTransform), With<AimingTracer>>,
        QueryState<(&Asteroid, &GlobalTransform)>
    )>
) {
    if let Some(base) = turn_state.active_base {
        let transform_res = query.q0().get(base);
        if let Ok(base_transform) = transform_res {
            let aim_rotation = base_transform.rotation * Quat::from_rotation_z(turn_state.firing_angle);
            let direction = aim_rotation * Vec3::new(0.0, 1.0 ,0.0);
            let thrust = Vec2::new(direction.x, direction.y) * turn_state.power;
            let offset = direction * 12.0;
            let base_pos3 = offset + base_transform.translation;
            let base_pos = Vec2::new(base_pos3.x, base_pos3.y);

            // Do in 2 stages for mutabilty
            let mut positions : HashMap<Entity, Vec2> = HashMap::new();
            for (entity, trace) in tracer_query.iter() {
                let position = calculate_position(&query.q2(), base_pos, trace.delay, thrust);
                positions.insert(entity, position);
            }
            for (entity, mut trace_transform) in query.q1().iter_mut() {
                if let Some(position) = positions.get(&entity) {
                    trace_transform.translation = Vec3::new(position.x, position.y, 0.0);
                }
            }
        }
    }
}

fn aiming_ui_aiming_end(
    mut commands: Commands,
    mut events : EventReader<TurnFiring>,
    tracer_query : Query<Entity, With<AimingTracer>>
) {
    for _ in events.iter() {
        for entity in tracer_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Plugins

pub struct AimingPlugin;

impl Plugin for AimingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AimingMaterials>()
           .add_system_set(
            SystemSet::on_update(AppState::InGame)
              .with_system(aiming_system.system())
              .with_system(aiming_ui_aiming_start.system())
              .with_system(aiming_ui_aiming_end.system())
              .with_system(aiming_ui_update_system.system())
        );
    }
}