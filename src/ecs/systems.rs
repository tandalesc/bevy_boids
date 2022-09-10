use bevy::prelude::*;

use crate::util::rect::transform_to_rect;

use super::{
    components::{Boid, Velocity},
    resources::{EntityQuadtree, EntityWrapper},
};

pub fn update_translation(
    time: Res<Time>,
    mut boid_query: Query<(&Velocity, &mut Transform), With<Boid>>,
) {
    for (velocity, mut transform) in &mut boid_query {
        let dv = velocity.0 * time.delta_seconds();
        transform.translation += dv;
    }
}

pub fn update_quadtree(
    entity_query: Query<(Entity, &Transform), With<Boid>>,
    mut quadtree: ResMut<EntityQuadtree>,
) {
    for (entity, transform) in &entity_query {
        let rect = transform_to_rect(transform);
        let entity_wrapper = EntityWrapper { entity, rect };
        if let Some(node) = quadtree.query_value_mut(&entity_wrapper) {
            node.delete(&entity_wrapper);
            quadtree.add(entity_wrapper);
            quadtree.debug();
        }
    }
}

pub fn update_velocity(
    time: Res<Time>,
    mut velocity_query: Query<(&mut Velocity, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    for (mut velocity, entity, transform) in &mut velocity_query {
        let rect = transform_to_rect(transform);
        let value = EntityWrapper { entity, rect };
        //collect distance to nearby boids
        let mut distances = vec![];
        if let Some(node) = quadtree.query_value(&value) {
            for value in &node.values {
                distances.push(value.rect.min.distance(node.rect.min));
            }
        }
        velocity.0 += Vec3::NEG_Y * 9.8 * time.delta_seconds();
    }
}
