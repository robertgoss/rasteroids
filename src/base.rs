use bevy::prelude::*;

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


pub struct PercentBar {
    pub val : f32,
    pub size : f32
}

pub fn add_base(
    commands: &mut Commands, angle : f32, 
    texture : Handle<ColorMaterial>, 
    base_health_back : Handle<ColorMaterial>, 
    base_health_front : Handle<ColorMaterial>, 
    asteroid : Entity, 
    player : Entity
) -> Entity {
    let health_bar = commands.spawn_bundle(SpriteBundle {
        material: base_health_front,
        transform: Transform::from_xyz(0.0, 30.0, 0.0),
        sprite: Sprite::new(Vec2::new(45.0, 4.0)),
        ..Default::default()
    }).insert(PercentBar { val : 1.0, size : 45.0}).id();
    let base = commands.spawn_bundle(SpriteBundle {
        material: texture,
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
    ).with_children(|base_builder| {
        base_builder.spawn_bundle(SpriteBundle {
            material: base_health_back,
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
                println!("Damage {}, {}", damage, dist);
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

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<BaseDestroyed>()
            .add_system(percent_update.system())
           .add_system(damage_base.system())
           .add_system(destroy_base.system());
    }
}