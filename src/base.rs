use bevy::prelude::*;

use super::explosion::Explode;

pub struct Base {
    pub angle : f32,
    pub offset : f32,
    pub health : f32
}

pub struct BaseOwner {
    pub entity : Entity
}

pub struct BaseDestroyed { 
    pub base : Entity
}


pub fn add_base(commands: &mut Commands, angle : f32, texture : Handle<ColorMaterial>, asteroid : Entity, player : Entity) -> Entity {
    commands.spawn_bundle(SpriteBundle {
        material: texture,
        transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).insert(Base{
        angle : angle,
        offset : -8.5,
        health : 100.0
    }).insert(Parent(asteroid)
    ).insert(BaseOwner{entity : player}
    ).id()
}

fn damage_base(
    mut bases : Query<(&mut Base, &Transform, Entity)>,
    mut events : EventReader<Explode>,
    mut event_destroy : EventWriter<BaseDestroyed>
) {
    let max_dist = 100.0;
    for event in events.iter() {
        let pos = Vec3::new(event.pos.x, event.pos.y, 0.0);
        for (mut base, transform, entity) in bases.iter_mut() {
            let dist = 1.0_f32.max(transform.translation.distance(pos) - 25.0); // Correct for shell
            if dist < max_dist {
                let damage = event.power * 1.0 / dist;
                base.health -= damage;
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

pub struct BasePlugin;

impl Plugin for BasePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<BaseDestroyed>()
           .add_system(damage_base.system())
           .add_system(destroy_base.system());
    }
}