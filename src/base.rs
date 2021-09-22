use bevy::prelude::*;

use crate::turn::TurnStart;

use super::explosion::Explode;
pub struct Base {
    pub angle : f32,
    pub offset : f32,
    pub health : f32,
    pub health_bar : Entity
}

pub struct BaseOwner {
    pub entity : Entity
}

pub struct BaseDestroyed { 
    pub base : Entity
}

pub struct BaseActivity {
    pub active : bool
}

pub struct BaseMaterials {
    base : Handle<ColorMaterial>,
    base_active : Handle<ColorMaterial>,
    base_bar_background : Handle<ColorMaterial>
}

impl FromWorld for BaseMaterials {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let base_handle = asset_server.load("images/base.png");
        let base_active_handle = asset_server.load("images/base_highlight.png");
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        BaseMaterials {
            base : materials.add(base_handle.into()),
            base_active : materials.add(base_active_handle.into()),
            base_bar_background : materials.add(Color::rgb(0.15, 0.15, 0.15).into())
        }
    }
}

pub struct PercentBar {
    pub val : f32,
    pub size : f32
}

pub fn add_base(
    commands: &mut Commands, angle : f32, 
    materials : &BaseMaterials,
    asteroid : Entity, 
    player : Entity,
    player_colour : Handle<ColorMaterial>
) -> Entity {
    let health_bar = commands.spawn_bundle(SpriteBundle {
        material: player_colour,
        transform: Transform::from_xyz(0.0, 30.0, 0.0),
        sprite: Sprite::new(Vec2::new(45.0, 4.0)),
        ..Default::default()
    }).insert(PercentBar { val : 1.0, size : 45.0}).id();

    let base = commands.spawn_bundle(SpriteBundle {
        material: materials.base.clone(),
        transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).insert(Base{
        angle : angle,
        offset : -8.5,
        health : 100.0,
        health_bar : health_bar
    }).insert(Parent(asteroid)
    ).insert(BaseOwner{entity : player}
    ).insert(BaseActivity{ active : false }
    ).with_children(|base_builder| {
        base_builder.spawn_bundle(SpriteBundle {
            material: materials.base_bar_background.clone(),
            transform: Transform::from_xyz(0.0, 30.0, 0.0),
            sprite: Sprite::new(Vec2::new(45.0, 5.0)),
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
        sprite.size.x = bar.val * bar.size;
        transform.translation.x = 0.5 * bar.size * (bar.val - 1.0);
    }
}

fn base_activity_changed (
    mut base_query : Query<(&BaseActivity, &mut Handle<ColorMaterial>),Changed<BaseActivity>>,
    materials : Res<BaseMaterials>
) {
    for (activity, mut material) in base_query.iter_mut() {
        if activity.active {
            *material = materials.base_active.clone();
        } else {
            *material = materials.base.clone();
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

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<BaseDestroyed>()
           .init_resource::<BaseMaterials>()
           .add_system(percent_update.system())
           .add_system(base_new_turn.system())
           .add_system(base_activity_changed.system())
           .add_system(damage_base.system())
           .add_system(destroy_base.system());
    }
}