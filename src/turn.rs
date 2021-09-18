use bevy::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub enum TurnPhase {
    Aiming, 
    Firing,
    Paused
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
pub struct TurnStart {
    pub new_base : Entity
}

pub struct TurnEnd;

pub struct TurnFiring;

fn turn_firing_phase_start( 
    mut turn_state : ResMut<TurnState>,
    mut events : EventReader<TurnFiring>
) {
    for _ in events.iter() {
        turn_state.phase = TurnPhase::Firing;
    }
}

fn turn_starter(
    mut turn_state : ResMut<TurnState>,
    mut events : EventReader<TurnStart>
)
{
    for event in events.iter() {
        turn_state.phase = TurnPhase::Aiming;
        turn_state.active_base = Some(event.new_base);
        turn_state.firing_angle = 0.0;
        turn_state.power = 70.0;
    }
}

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<TurnStart>()
           .add_event::<TurnEnd>()
           .add_event::<TurnFiring>()
           .init_resource::<TurnState>()
           .add_system(turn_starter.system())
           .add_system(turn_firing_phase_start.system());
    }
}