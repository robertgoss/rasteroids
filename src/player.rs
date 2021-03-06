use bevy::prelude::*;

use std::collections::BTreeSet;

use super::base::{BaseOwner,BaseDestroyed};
use super::turn::{TurnStart, TurnEnd};
use super::app_state::AppState;
use super::victory_menu::Victory;

#[derive(Default)]
pub struct PlayerOrder {
    pub order : Vec<Entity>,
    pub current : usize
}

#[derive(Clone, Component)]
pub struct Player {
    pub name : String,
    bases : BTreeSet<Entity>,
    current_base : usize,
    pub colour : Color
}

pub fn setup_players(
    commands : &mut Commands,
    player_order : &mut PlayerOrder,
    font : Handle<Font>
) -> Vec<Player> {
    let players : Vec<Player> = [
        Player {
            name : "Robert".to_string(), 
            bases : BTreeSet::new(), 
            current_base: 0,
            colour : Color::rgb(0.75, 0.15, 0.15).into(),
        },
        Player {
            name : "James".to_string(), 
            bases : BTreeSet::new(), 
            current_base: 0,
            colour : Color::rgb(0.15, 0.75, 0.15).into(),
        }
    ].to_vec();
    let player_1 = commands.spawn().insert(players[0].clone() ).id();
    let player_2 = commands.spawn().insert(players[1].clone() ).id();
    player_order.order.push(player_1);
    player_order.order.push(player_2);
    player_order.current = 0;

    setup_player_ui(commands, &players, font);

    players
}

#[derive(Component)]
struct PlayerUI {
    index : usize
}

fn setup_player_ui (
    commands : &mut Commands,
    players : &Vec<Player>,
    font : Handle<Font>
) {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            size: Size::new(Val::Px(200.0), Val::Auto),
            flex_direction : FlexDirection::Column,
            position_type: PositionType::Absolute,
            position: Rect {
                right: Val::Px(20.0),
                top: Val::Px(20.0),
                ..Default::default()
            },
            border: Rect::all(Val::Px(10.0)),
            ..Default::default()
        },
        color: Color::rgba(0.4, 0.4, 0.4, 0.8).into(),
        ..Default::default()
    })
    .with_children(|parent| {
        for (index,player) in players.iter().enumerate() {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    player.name.clone(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 40.0,
                        color: player.colour,
                    },
                    Default::default(),
                ),
                ..Default::default()
            }).insert(PlayerUI {index : index} );
        }
    });
}

fn player_ui_current(
    mut player_ui_query : Query<(&PlayerUI, &mut Text)>,
    player_order : Res<PlayerOrder>,
) {
    for (player_ui, mut text) in player_ui_query.iter_mut() {
        if let Some(mut section) = text.sections.first_mut() {
            if player_ui.index == player_order.current {
                section.style.font_size = 50.0;
            } else {
                section.style.font_size = 35.0;
            }
        }
    }
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
    mut victory : ResMut<Victory>,
    mut state: ResMut<State<AppState>>
) {
    for event in events.iter() {
        for mut player in player_query.iter_mut() {
            player.bases.remove(&event.base);
        }
        let active_count = player_query.iter_mut().filter(
            |player| !player.bases.is_empty()
        ).count();
        if active_count <= 1 {
            victory.player = player_query.iter_mut().filter(
                |player| !player.bases.is_empty()
            ).map(|p| p.clone()
            ).next();
            state.set(AppState::VictoryMenu).unwrap();
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

fn teardown_players(
    mut commands : Commands,
    player_query : Query<Entity, With<Player>>,
    player_ui_query : Query<Entity, With<PlayerUI>>,
    mut player_order : ResMut<PlayerOrder>
) {
    for player in player_query.iter() {
        commands.entity(player).despawn_recursive();
    }
    for player_ui in player_ui_query.iter() {
        commands.entity(player_ui).despawn_recursive();
    }
    player_order.order.clear();
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerOrder>()
           .add_system_set(
              SystemSet::on_update(AppState::InGame)
                .with_system(base_owned.system())
                .with_system(base_lost.system())
                .with_system(next_turn.system())
                .with_system(player_ui_current.system())
              )
           .add_system_set(
              SystemSet::on_exit(AppState::InGame)
               .with_system(teardown_players.system())
           );
    }
}