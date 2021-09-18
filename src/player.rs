use bevy::prelude::*;

use super::base::BaseOwner;
use super::turn::{TurnStart, TurnEnd};


#[derive(Default)]
pub struct PlayerOrder {
    pub order : Vec<Entity>,
    pub current : usize
}

pub struct Player {
    name : String,
    bases : Vec<Entity>,
    current_base : usize
}

pub fn setup_players(
    commands : &mut Commands,
    player_order : &mut PlayerOrder
) {
    let player_1 = commands.spawn().insert(
        Player {name : "Robert".to_string(), bases : Vec::new(), current_base: 0}
    ).id();
    let player_2 = commands.spawn().insert(
        Player {name : "James".to_string(), bases : Vec::new(), current_base: 0}
    ).id();
    player_order.order.push(player_1);
    player_order.order.push(player_2);
    player_order.current = 0;
}

fn base_owned(
    ownership_added : Query<(Entity, &BaseOwner), Added<BaseOwner>>,
    mut player_query : Query<&mut Player>,
) {
    for (base, owner) in ownership_added.iter() {
        if let Ok(mut player) = player_query.get_mut(owner.entity) {
            player.bases.push(base);
        }
    }
}

fn next_turn(
    mut events_end : EventReader<TurnEnd>,
    mut events_start : EventWriter<TurnStart>,
    mut player_order : ResMut<PlayerOrder>,
    mut player_query : Query<&mut Player>
) {
    for _ in events_end.iter() {
        player_order.current += 1;
        if player_order.current >= player_order.order.len() {
            player_order.current = 0;
        }
        let mut player = player_query.get_mut(player_order.order[player_order.current]).unwrap();
        player.current_base += 1;
        if player.current_base >= player.bases.len() {
            player.current_base = 0;
        }
        events_start.send(TurnStart{ new_base : player.bases[player.current_base] });
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PlayerOrder>()
           .add_system(base_owned.system())
           .add_system(next_turn.system());
    }
}