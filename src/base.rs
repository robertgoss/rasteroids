use bevy::prelude::*;

pub struct Base {
    pub angle : f32,
    pub offset : f32
}

pub struct BaseOwner {
    pub entity : Entity
}

pub fn add_base(commands: &mut Commands, angle : f32, texture : Handle<ColorMaterial>, asteroid : Entity, player : Entity) -> Entity {
    commands.spawn_bundle(SpriteBundle {
        material: texture,
        transform: Transform::from_rotation(Quat::from_rotation_z(angle)),
        sprite: Sprite::new(Vec2::new(50.0, 50.0)),
        ..Default::default()
    }).insert(Base{
        angle : angle,
        offset : -8.5
    }).insert(Parent(asteroid)
    ).insert(BaseOwner{entity : player}
    ).id()
}