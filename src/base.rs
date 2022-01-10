use bevy::prelude::*;

use super::turn::TurnStart;
use super::explosion::Explode;
use super::app_state::AppState;
use super::asteroids::Asteroid;

#[derive(Component)]
pub struct Base {
    pub angle : f32,
    pub offset : f32,
    pub health : f32,
    pub health_bar : Entity
}

#[derive(Component)]
pub struct BaseOwner {
    pub entity : Entity
}

pub struct BaseDestroyed { 
    pub base : Entity
}

#[derive(Component)]
pub struct BaseActivity {
    pub active : bool
}

pub struct BaseTextures {
    base : Handle<Image>,
    base_active : Handle<Image>,
    base_bar_background : Color
}

impl FromWorld for BaseTextures {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        BaseTextures {
            base : asset_server.load("images/base.png"),
            base_active : asset_server.load("images/base_highlight.png"),
            base_bar_background : Color::rgb(0.15, 0.15, 0.15)
        }
    }
}

#[derive(Component)]
pub struct PercentBar {
    pub val : f32,
    pub size : f32
}

pub fn add_base(
    commands: &mut Commands, angle : f32, 
    textures : &BaseTextures,
    asteroid : &(Entity, Asteroid), 
    player : Entity,
    player_colour : Color
) -> Entity {
    let health_bar = commands.spawn_bundle(SpriteBundle {
        transform: Transform::from_xyz(0.0, 30.0, 0.0),
        sprite: Sprite { 
            custom_size : Some(Vec2::new(45.0, 4.0)),
            color : player_colour,
            ..Default::default() 
        },
        ..Default::default()
    }).insert(PercentBar { val : 1.0, size : 45.0}).id();

    let offset = -8.5;
    let radius = asteroid.1.radius - offset;
    let pos = Vec3::new(-radius * angle.sin(), radius * angle.cos(), 0.0);

    let base = commands.spawn_bundle(SpriteBundle {
        texture: textures.base.clone(),
        transform: Transform {
            rotation : Quat::from_rotation_z(angle),
            translation : pos,
            scale : Vec3::new(1.0, 1.0, 1.0)
        },
        sprite: Sprite { 
            custom_size : Some(Vec2::new(50.0, 50.0)),
            ..Default::default() 
        },
        ..Default::default()
    }).insert(Base{
        angle : angle,
        offset : offset,
        health : 100.0,
        health_bar : health_bar
    }).insert(Parent(asteroid.0)
    ).insert(BaseOwner{entity : player}
    ).insert(BaseActivity{ active : false }
    ).with_children(|base_builder| {
        base_builder.spawn_bundle(SpriteBundle {
            sprite: Sprite { 
                custom_size : Some(Vec2::new(45.0, 5.0)),
                color : textures.base_bar_background,
                ..Default::default() 
            },
            transform: Transform::from_xyz(0.0, 30.0, 0.0),
            ..Default::default()
        });
    }).id();
    commands.entity(base).push_children(&[health_bar]);
    base
}

fn damage_base(
    mut bases : Query<(&mut Base, &GlobalTransform, Entity)>,
    mut percent_query : Query<&mut PercentBar>,
    mut events : EventReader<Explode>,
    mut event_destroy : EventWriter<BaseDestroyed>
) {
    let max_dist = 20.0;
    for event in events.iter() {
        let pos = Vec3::new(event.pos.x, event.pos.y, 0.0);
        for (mut base, transform, entity) in bases.iter_mut() {
            let dist = 1.0_f32.max(transform.translation.distance(pos) - 25.0); // Correct for shell
            if dist < max_dist {
                let damage = event.power * (max_dist - dist) / max_dist;
                base.health -= damage;
                let percent = base.health.max(0.0) / 100.0;
                if let Ok(mut bar) = percent_query.get_mut(base.health_bar) {
                    bar.val = percent;
                }
                if base.health < 0.0 {
                    event_destroy.send(BaseDestroyed {base : entity})
                }
            }
        }
    }
}

fn destroy_base(
    mut commands : Commands,
    mut events : EventReader<BaseDestroyed>,
) {
    for event in events.iter() {
        commands.entity(event.base).despawn_recursive();
    }
}

fn percent_update(
    mut percent_query : Query<(&PercentBar, &mut Sprite, &mut Transform), Changed<PercentBar>>
) {
    for (bar, mut sprite, mut transform) in percent_query.iter_mut() {
        sprite.custom_size = sprite.custom_size.map(
            |vec| Vec2::new(bar.val * bar.size, vec.y)
        );
        transform.translation.x = 0.5 * bar.size * (bar.val - 1.0);
    }
}

fn base_activity_changed (
    mut base_query : Query<(&BaseActivity, &mut Handle<Image>),Changed<BaseActivity>>,
    textures : Res<BaseTextures>
) {
    for (activity, mut texture) in base_query.iter_mut() {
        if activity.active {
            *texture = textures.base_active.clone();
        } else {
            *texture = textures.base.clone();
        }
    }
}

fn base_new_turn(
    mut events : EventReader<TurnStart>,
    mut base_activity : Query<(Entity, &mut BaseActivity)>
) {
    for event in events.iter() {
        for (entity, mut base_active) in base_activity.iter_mut() {
            if entity == event.new_base && !base_active.active  {
                base_active.active = true;
            } 
            if entity != event.new_base && base_active.active {
                base_active.active = false;
            }
        }
    }
}

fn teardown_bases(
    mut commands : Commands,
    base_query : Query<Entity, With<Base>>
) {
    for base in base_query.iter() {
        commands.entity(base).despawn_recursive();
    }
}

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BaseDestroyed>()
           .init_resource::<BaseTextures>()
           .add_system_set(
            SystemSet::on_update(AppState::InGame)
              .with_system(percent_update.system())
              .with_system(base_new_turn.system())
              .with_system(base_activity_changed.system())
              .with_system(damage_base.system())
              .with_system(destroy_base.system())
            )
            .add_system_set(
                SystemSet::on_exit(AppState::InGame)
                  .with_system(teardown_bases.system())
            );
    }
}