use bevy::prelude::*;

use super::collide::Circle;
use super::base::Base;

pub struct AsteroidDrawable;
pub struct Asteroid{
    pub max_radius : f32,
    pub radius : f32
}

impl Asteroid {
    pub fn bound(self : &Self, transform : &GlobalTransform) -> Circle {
        let centre = Vec2::new(transform.translation.x, transform.translation.y);
        Circle { radius : self.radius, centre : centre }
    }
}

pub fn add_asteroid(commands: &mut Commands, x : f32, y : f32, texture : Handle<ColorMaterial>) -> Entity {
    let max_radius = 100.0;
    commands.spawn().insert(Asteroid{
        max_radius : 100.0,
        radius : 50.0
    }).insert(Transform::from_xyz(x, y, 0.0)
    ).insert(GlobalTransform::from_xyz(x, y, 0.0)
    ).with_children(
        |parent| {
            parent.spawn_bundle(SpriteBundle {
                material: texture,
                transform: Transform::identity(),
                sprite: Sprite::new(Vec2::new(2.0*max_radius, 2.0*max_radius)),
                ..Default::default()
            }).insert(AsteroidDrawable);
        }
    ).id()
}

// If an asteroid's radius changes we want to update it's sprite and reposition bases on the surface
// so they stay on the surface
pub fn asteroid_changed(
    query: Query<(&Asteroid, &Children), 
    Changed<Asteroid>>,
    mut transform_query: Query<&mut Transform>,
    bases_query: Query<&Base>,
    asteroid_drawable_query: Query<&AsteroidDrawable>
) {
    for (asteroid, children) in query.iter() {
        // Reposition any bases and update the drawable
        for child in children.iter() {
            // If this is a base reposition it
            if let Ok(base) = bases_query.get(*child) {
                if let Ok(mut transform) = transform_query.get_mut(*child) {
                    let angle = base.angle;
                    let radius = asteroid.radius - base.offset;
                    transform.translation = Vec3::new(-radius * angle.sin(), radius * angle.cos(), 0.0);
                }
            } 
            // Asteroid drawable children need to be scales to new size
            if asteroid_drawable_query.get(*child).is_ok() {
                if let Ok(mut transform) = transform_query.get_mut(*child) {
                    let scale = asteroid.radius / asteroid.max_radius;
                    transform.scale = Vec3::new(scale, scale, scale);
                }
            }
        }
    }
}