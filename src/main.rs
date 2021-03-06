#![feature(exact_size_is_empty)]

use bevy::prelude::*;

pub mod collide;
pub mod base;
pub mod asteroids;
pub mod weapon;
pub mod turn;
pub mod aiming;
pub mod player;
pub mod explosion;
pub mod app_state;
pub mod main_menu;
pub mod victory_menu;

use asteroids::{calculate_gravity, add_asteroid, AsteroidPlugin, Asteroid};
use base::{add_base, BasePlugin, BaseTextures};
use weapon::{Weapon, WeaponPlugin, WeaponType, Launch, WeaponExplode};
use turn::{TurnPlugin, TurnState, TurnStart, TurnFiring, TurnPhase};
use aiming::AimingPlugin;
use player::{setup_players, PlayerOrder, PlayerPlugin};
use explosion::ExplosionPlugin;
use app_state::AppState;
use main_menu::MainMenuPlugin;
use victory_menu::VictoryMenuPlugin;

#[derive(Component)]
struct Background;


fn setup(
    mut commands: Commands,
    mut player_order: ResMut<PlayerOrder>,
    base_materials : Res<BaseTextures>,
    asset_server: Res<AssetServer>,
    mut events : EventWriter<TurnStart>
) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    // Background sprite
    let star_map_handle = asset_server.load("images/starfield.png");
    commands.spawn_bundle(SpriteBundle {
        texture: star_map_handle,
        transform: Transform::from_scale(Vec3::new(1.5, 1.5, 1.5)),
        ..Default::default()
    }).insert(Background);
    // Players 
    let players = setup_players(
        &mut commands,
        &mut player_order,
        asset_server.load("fonts/FiraSans-Bold.ttf")
    );
    // Asteroids
    let asteroid_texture_handle = asset_server.load("images/pallas_asteroid_alpha.png");
    let asteroids = vec!( 
      add_asteroid(&mut commands, 0.0, -215.0, asteroid_texture_handle.clone()),
      add_asteroid(&mut commands, -60.0, 0.0, asteroid_texture_handle.clone()),
      add_asteroid(&mut commands, 60.0, 0.0, asteroid_texture_handle.clone())
    );
    // Bases
    let colours :Vec<Color> = players.iter().map(
        |player| player.colour
    ).collect();
    let base_1 = add_base( &mut commands, 0.0, &base_materials, &asteroids[0], player_order.order[0], colours[0].clone());
    add_base( &mut commands, 1.0, &base_materials, &asteroids[0], player_order.order[1], colours[1].clone());
    add_base( &mut commands, 2.0, &base_materials, &asteroids[1], player_order.order[0], colours[0].clone());
    add_base( &mut commands, 3.0, &base_materials, &asteroids[2], player_order.order[1], colours[1].clone());
    events.send(TurnStart{new_base : base_1});
}


fn firing_system(
    key_input: Res<Input<KeyCode>>,
    turn_state : Res<TurnState>,
    mut launch_events: EventWriter<Launch>,
    mut turn_events : EventWriter<TurnFiring>
) {
    if key_input.just_pressed(KeyCode::Space) && turn_state.phase == TurnPhase::Aiming {
        // Test launch a rockets
        if let Some(base) = turn_state.active_base {
            launch_events.send(Launch{
                angle : turn_state.firing_angle, 
                offset : 12.0, 
                thrust : turn_state.power, 
                parent : base, 
                weapon_type : WeaponType::Rocket
            });
            turn_events.send(TurnFiring);
        }
    }
}

fn gravity_system(
    mut rocket_query : Query<(&mut Weapon, &GlobalTransform)>,
    asteroid_query : Query<(&Asteroid, &GlobalTransform)>,
    time : Res<Time>
) {
    for (mut rocket, rocket_transform) in rocket_query.iter_mut() {
        let rocket_pos = rocket_transform.translation;
        let pos = Vec2::new(rocket_pos.x, rocket_pos.y);
        rocket.thrust +=  calculate_gravity(&asteroid_query, pos, time.delta_seconds());
    }
}

fn rocket_asteroid_collide_system(
    rocket_query : Query<(Entity, &Weapon, &GlobalTransform)>,
    asteroid_query : Query<(&Asteroid, &GlobalTransform)>,
    mut events: EventWriter<WeaponExplode>
) {
    for (entity, rocket, rocket_transform) in rocket_query.iter() {
        for (asteroid, asteroid_transform) in asteroid_query.iter() {
            if rocket.bound(rocket_transform).collide(asteroid.bound(asteroid_transform)) {
                events.send(WeaponExplode { entity : entity })
            }
        }
    }
}

fn teardown_ingame(
    mut commands : Commands,
    background_query : Query<Entity, With<Background>>
) {
    for background in background_query.iter() {
        commands.entity(background).despawn_recursive();
    }
}

fn main() {
    App::new().insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
              .add_plugins(DefaultPlugins)
              .add_state(AppState::MainMenu)
              .add_plugin(MainMenuPlugin)
              .add_system_set(
                  SystemSet::on_enter(AppState::InGame)
                  .with_system(setup)
              )
              .add_system_set(
                  SystemSet::on_update(AppState::InGame)
                  .with_system(firing_system.system())
                  .with_system(gravity_system.system())
                  .with_system(rocket_asteroid_collide_system.system())
              )
              .add_system_set(
                 SystemSet::on_exit(AppState::InGame)
                 .with_system(teardown_ingame.system())
              )
              .add_plugin(PlayerPlugin)
              .add_plugin(AsteroidPlugin)
              .add_plugin(WeaponPlugin)
              .add_plugin(TurnPlugin)
              .add_plugin(AimingPlugin)
              .add_plugin(ExplosionPlugin)
              .add_plugin(BasePlugin)
              .add_plugin(VictoryMenuPlugin)
              .run();
}
