use bevy::prelude::*;

use crate::util::quadtree::QuadtreeStats;

use super::{
    components::{Boid, Velocity},
    resources::{EntityQuadtree, EntityWrapper},
};

pub fn apply_kinematics(
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
        let value = EntityWrapper::new(entity, transform);
        quadtree.delete(&value);
        quadtree.add(value);
    }
    quadtree.clean_structure();
    QuadtreeStats::calculate(&quadtree).print();
}

pub fn avoid_nearby_boids(
    mut velocity_query: Query<(&mut Velocity, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    for (mut velocity, entity, transform) in &mut velocity_query {
        let my_value = EntityWrapper::new(entity, transform);
        let my_diag = my_value.rect.max - my_value.rect.min;
        let my_midpoint = my_value.rect.min + my_diag / 2.;
        let mut velocity_correction = Vec3::new(0., 0., 0.);
        if let Some(node) = quadtree.query_value(&my_value) {
            let num_values = node.values.len();
            for value in &node.values {
                let diag = value.rect.max - value.rect.min;
                let midpoint = value.rect.min + diag / 2.;
                let distance = midpoint.distance(my_midpoint.clone());
                let direction_away = (midpoint - my_midpoint).normalize_or_zero().extend(0.);
                velocity_correction += direction_away / (1. + distance);
            }
            if num_values > 0 {
                velocity_correction /= num_values as f32;
            }
        }
        if velocity_correction.length_squared() > 0.0000001 {
            velocity.0 -= velocity_correction;
        }
    }
}
