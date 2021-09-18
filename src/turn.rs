use bevy::prelude::*;

use super::weapon::ActiveWeapon;

#[derive(PartialEq, Eq, Debug)]
pub enum TurnPhase {
    Aiming, 
    Firing
}

impl Default for TurnPhase {
    fn default() -> Self {
        TurnPhase::Aiming
    }
}

#[derive(Default)]
pub struct TurnState {
    pub phase : TurnPhase,
    pub active_base : Option<Entity>,
    pub firing_angle : f32,
    pub power : f32
}

// Event
pub struct TurnPhaseChanged {
    pub new_phase : TurnPhase
}


fn turn_phase_update( 
    mut turn_state : ResMut<TurnState>, 
    weapon_query : Query<&ActiveWeapon>,
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




pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<TurnPhaseChanged>()
           .init_resource::<TurnState>()
           .add_startup_system(turn_setup.system())
           .add_system(turn_phase_update.system());
    }
}