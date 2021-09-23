use bevy::prelude::*;

use super::collide::Circle;
use super::base::{Base, BaseDestroyed};
use super::explosion::Explode;
use super::app_state::AppState;

pub struct AsteroidDrawable;

#[derive(Clone)]
pub struct Asteroid{
    pub max_radius : f32,
    pub radius : f32
}

pub struct AsteroidDestroyed { 
    pub asteroid : Entity
}

impl Asteroid {
    pub fn bound(self : &Self, transform : &GlobalTransform) -> Circle {
        let centre = Vec2::new(transform.translation.x, transform.translation.y);
        Circle { radius : self.radius, centre : centre }
    }
}

pub fn add_asteroid(commands: &mut Commands, x : f32, y : f32, texture : Handle<ColorMaterial>) -> (Entity, Asteroid) {
    let max_radius = 100.0;
    let asteroid = Asteroid{
        max_radius : 100.0,
        radius : 50.0
    };
    let id = commands.spawn().insert(asteroid.clone()).insert(Transform::from_xyz(x, y, 0.0)
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
    ).id();
    (id, asteroid)
}

// If an asteroid's radius changes we want to update it's sprite and reposition bases on the surface
// so they stay on the surface
pub fn asteroid_changed(
    query: Query<(&Asteroid, &Children), Changed<Asteroid>>,
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
                    println!("base moved");
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
        println!("Changed");
    }
}

fn damage_asteroid(
    mut asteroids : Query<(&mut Asteroid, &GlobalTransform, Entity)>,
    mut events : EventReader<Explode>,
    mut event_destroy : EventWriter<AsteroidDestroyed>
) {
    let max_dist = 20.0;
    let min_radius = 8.0;
    for event in events.iter() {
        let pos = Vec3::new(event.pos.x, event.pos.y, 0.0);
        for (mut asteroid, transform, entity) in asteroids.iter_mut() {
            let dist = 1.0_f32.max(transform.translation.distance(pos) - asteroid.radius); // Correct for shell
            if dist < max_dist {
                let damage = event.power * (max_dist - dist) / max_dist;
                if asteroid.radius - damage < min_radius {
                    event_destroy.send(AsteroidDestroyed {asteroid : entity})
                } else {
                    asteroid.radius -= damage;
                }
            }
        }
    }
}

fn destroy_asteroid(
    mut commands : Commands,
    mut events : EventReader<AsteroidDestroyed>,
    child_query : Query<&Children, With<Asteroid>>,
    base_query : Query<Entity, With<Base>>,
    mut base_destroy : EventWriter<BaseDestroyed>
) {
    for event in events.iter() {
        // Destroy children (send event to bases to let them destroy themselves)
        if let Ok(children) = child_query.get(event.asteroid) {
            for child in children.iter() {
                if base_query.get(*child).is_ok() {
                    base_destroy.send(BaseDestroyed { base : *child});
                } else {
                    commands.entity(*child).despawn_recursive();
                }
            }
        }
        // Destroy asteroid now children dealt with
        commands.entity(event.asteroid).despawn();
    }
}

// Plugin
pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<AsteroidDestroyed>()
           .add_system_set(
              SystemSet::on_update(AppState::InGame)
                .with_system(destroy_asteroid.system())
                .with_system(damage_asteroid.system())
                .with_system(asteroid_changed.system())
            );
    }
}