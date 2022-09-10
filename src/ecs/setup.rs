use bevy::{prelude::*, sprite::Rect};

use crate::util::quadtree::QuadtreeStats;

use super::{
    components::{Boid, Collider, Velocity},
    resources::{EntityQuadtree, EntityWrapper},
};

/* Public Functions */

pub fn spawn_boids(mut commands: Commands, mut quadtree: ResMut<EntityQuadtree>) {
    let scale = Vec3::new(3.5, 3.5, 0.);
    let count = (50, 50);
    let spacing = (15., 10.);
    // create (count.0 * count.1) boids
    for x_i32 in 0..count.0 {
        for y_i32 in 0..count.1 {
            // center boids on screen
            let x = (x_i32 as f32) * spacing.0 - count.0 as f32/2. * spacing.0;
            let y = (y_i32 as f32) * spacing.1 - count.1 as f32/2. * spacing.1;
            let translation = Vec3::new(x, y, 0.);
            let rect = Rect {
                min: Vec2::new(x, y),
                max: Vec2::new(x + scale.x, y + scale.y),
            };
            //spawn boid
            let entity = commands
                .spawn()
                .insert(Boid)
                .insert(Velocity(Vec3::NEG_X * 8.))
                .insert_bundle(create_boid_sprite(translation, scale.clone()))
                .insert(Collider)
                .id();
            //add to quadtree
            quadtree.add(EntityWrapper { entity, rect });
        }
    }
    QuadtreeStats::calculate(&quadtree).print();
}

pub fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

/* Internal-only Functions */

fn create_boid_sprite(translation: Vec3, scale: Vec3) -> SpriteBundle {
    SpriteBundle {
        transform: Transform {
            scale,
            translation,
            ..default()
        },
        sprite: Sprite {
            color: Color::AQUAMARINE,
            ..default()
        },
        ..default()
    }
}
