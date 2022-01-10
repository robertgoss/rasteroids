use bevy::prelude::*;

pub struct Explode {
    pub pos : Vec2,
    pub power : f32
}


pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Explode>();
    }
}