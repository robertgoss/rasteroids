use bevy::prelude::*;
use bevy::app::AppExit;

use std::collections::BTreeSet;

use super::base::{BaseOwner,BaseDestroyed};
use super::turn::{TurnStart, TurnEnd};


#[derive(Default)]
pub struct PlayerOrder {
    pub order : Vec<Entity>,
    pub current : usize
}

#[derive(Clone)]
pub struct Player {
    pub name : String,
    bases : BTreeSet<Entity>,
    current_base : usize,
    pub colour : Handle<ColorMaterial>
}

pub fn setup_players(
    commands : &mut Commands,
    materials : &mut Assets<ColorMaterial>,
    player_order : &mut PlayerOrder
) -> Vec<Player> {
    let players : Vec<Player> = [
        Player {
            name : "James".to_string(), 
            bases : BTreeSet::new(), 
            current_base: 0,
            colour : materials.add(Color::rgb(0.75, 0.15, 0.15).into()),
        },
        Player {
            name : "James".to_string(), 
            bases : BTreeSet::new(), 
            current_base: 0,
            colour : materials.add(Color::rgb(0.15, 0.75, 0.15).into()),
        }
    ].to_vec();
    let player_1 = commands.spawn().insert(players[0].clone() ).id();
    let player_2 = commands.spawn().insert(players[1].clone() ).id();
    player_order.order.push(player_1);
    player_order.order.push(player_2);
    player_order.current = 0;
    players
}

fn base_owned(
    ownership_added : Query<(Entity, &BaseOwner), Added<BaseOwner>>,
    mut player_query : Query<&mut Player>,
) {
    for (base, owner) in ownership_added.iter() {
        if let Ok(mut player) = player_query.get_mut(owner.entity) {
            player.bases.insert(base);
        }
    }
}

fn base_lost(
    mut events : EventReader<BaseDestroyed>,
    mut player_query : Query<&mut Player>,
    mut exit: EventWriter<AppExit>
) {
    for event in events.iter() {
        for mut player in player_query.iter_mut() {
            player.bases.remove(&event.base);
            if player.bases.is_empty() {
                // GameOver
                exit.send(AppExit);
            }
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
        let next_base = *player.bases.iter().nth(player.current_base).unwrap();
        events_start.send(TurnStart{ new_base :  next_base});
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<PlayerOrder>()
           .add_system(base_owned.system())
           .add_system(base_lost.system())
           .add_system(next_turn.system());
    }
}