use bevy::prelude::*;

use super::turn::{TurnPhase, TurnState, TurnStart, TurnFiring};
use super::weapon::{Launch, WeaponType, WeaponTracer};

// Components
struct AimingTracerTimer;


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
    mut events : EventReader<TurnStart>
) {
    for _ in events.iter() {
        commands.spawn().insert(AimingTracerTimer)
                        .insert(Timer::from_seconds(0.15, true));
            
    }
}

fn aiming_ui_timer_system(
    time: Res<Time>, 
    turn_state : Res<TurnState>,
    mut timer_query: Query<&mut Timer, With<AimingTracerTimer>>,
    mut events: EventWriter<Launch>
) {
    for mut timer in timer_query.iter_mut() {
        if timer.tick(time.delta()).just_finished() {
            if let Some(base) = turn_state.active_base {
                events.send(Launch{
                    angle : turn_state.firing_angle, 
                    offset : 10.0, 
                    thrust : turn_state.power, 
                    parent : base, 
                    weapon_type : WeaponType::Tracer
                });
            }
        }
    }
}

fn aiming_ui_aiming_end(
    mut commands: Commands,
    mut events : EventReader<TurnFiring>,
    timer_query : Query<Entity, With<AimingTracerTimer>>,
    tracer_query : Query<Entity, With<WeaponTracer>>
) {
    for _ in events.iter() {
        for entity in timer_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
        for entity in tracer_query.iter() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

// Plugins

pub struct AimingPlugin;

impl Plugin for AimingPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(aiming_system.system())
           .add_system(aiming_ui_aiming_start.system())
           .add_system(aiming_ui_aiming_end.system())
           .add_system(aiming_ui_timer_system.system());
    }
}