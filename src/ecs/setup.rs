use bevy::{prelude::*, sprite::Rect};
use rand::prelude::*;

use crate::util::quadtree::quadtree_stats::QuadtreeStats;

use super::{
    components::{Boid, Collider, Kinematics},
    resources::{EntityQuadtree, EntityWrapper},
};

pub const BOID_SCALE: Vec2 = Vec2::new(2.5, 2.5);
pub const BOID_COUNT: IVec2 = IVec2::new(50, 50);
pub const BOID_SPAWN_SPACING: Vec2 = Vec2::new(12., 6.);
pub const BOID_SPAWN_OFFSET: Vec2 = Vec2::new(
    BOID_COUNT.x as f32 * BOID_SPAWN_SPACING.x / 2.,
    BOID_COUNT.y as f32 * BOID_SPAWN_SPACING.y / 2.,
);

/* Public Functions */

pub fn spawn_boids(mut commands: Commands, mut quadtree: ResMut<EntityQuadtree>) {
    let mut rng = rand::thread_rng();
    // create (count.x * count.y) boids
    for x_i32 in 0..BOID_COUNT.x {
        for y_i32 in 0..BOID_COUNT.y {
            // center boids on screen
            let translation = Vec2::new(
                (x_i32 as f32) * BOID_SPAWN_SPACING.x - BOID_SPAWN_OFFSET.x,
                (y_i32 as f32) * BOID_SPAWN_SPACING.y - BOID_SPAWN_OFFSET.y,
            );
            let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0))
                .normalize_or_zero()
                .extend(0.)
                * 100.;
            //spawn boid
            let entity = commands
                .spawn()
                .insert(Boid)
                .insert(Kinematics {
                    velocity: velocity.clone(),
                    acceleration: Vec3::ZERO,
                })
                .insert(Collider)
                .insert_bundle(create_boid_sprite(
                    translation.extend(0.),
                    BOID_SCALE.extend(0.),
                ))
                .id();
            //add to quadtree
            let rect = Rect {
                min: translation.clone(),
                max: translation + BOID_SCALE,
            };
            quadtree.add(EntityWrapper {
                entity,
                rect,
                velocity,
            });
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
