use bevy::prelude::*;

use crate::util::{
    quadtree::{quadtree_stats::QuadtreeStats, quadtree_value::QuadtreeValue},
    rect::magnify_rect,
};

use super::{
    components::{Boid, Velocity},
    resources::{EntityQuadtree, EntityWrapper},
    setup::BOID_SCALE,
    PHYSICS_FRAME_RATE,
};

const EPS: f32 = 0.0000001;
const DELTA_TIME_FIXED: f32 = 1. / PHYSICS_FRAME_RATE as f32;
const BOID_DETECTION_RADIUS: f32 = 7.5;
const BOID_AVOIDANCE_FORCE: f32 = 12.;
const BOID_WALL_AVOIDANCE_FORCE: f32 = 24.;

pub fn apply_kinematics(
    // time: Res<Time>,
    mut boid_query: Query<(&Velocity, &mut Transform), With<Boid>>,
) {
    boid_query.par_for_each_mut(32, |(velocity, mut transform)| {
        // RK4
        let y0 = transform.translation;
        let h = DELTA_TIME_FIXED;
        let k1 = velocity.0;
        let k2 = ((y0 + k1 * (h / 2.)) - y0) / (h / 2.);
        let k3 = ((y0 + k2 * (h / 2.)) - y0) / (h / 2.);
        let k4 = ((y0 + k3 * h) - y0) / h;

        let dy = (k1 + (2. * k2) + (2. * k3) + k4) * h / 6.;
        transform.translation += dy;
    });
}

pub fn update_quadtree(
    entity_query: Query<(Entity, &Transform), With<Boid>>,
    mut quadtree: ResMut<EntityQuadtree>,
) {
    entity_query.for_each(|(entity, transform)| {
        // quadtrees have relatively fast delete and add operations, so just run that every time
        let value = EntityWrapper::new(entity, transform);
        if let Some(node) = quadtree.query_rect(value.get_rect()) {
            if !node.contains_value(&value) {
                quadtree.delete(&value);
                quadtree.add(value);
            }
        }
    });
    quadtree.clean_structure();
    // QuadtreeStats::calculate(&quadtree).print();
}

pub fn avoid_nearby_boids(
    mut velocity_query: Query<(&mut Velocity, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    velocity_query.par_for_each_mut(16, |(mut velocity, entity, transform)| {
        let my_value = EntityWrapper::new(entity, transform);
        let my_diag = my_value.rect.max - my_value.rect.min;
        let my_midpoint = my_value.rect.min + my_diag / 2.;
        let detection_rect = magnify_rect(my_value.get_rect(), Vec2::ONE * BOID_DETECTION_RADIUS);
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            if let Some(descendent_values) = node.get_all_descendant_values() {
                let mut velocity_correction = Vec3::new(0., 0., 0.);
                let num_values = descendent_values.len();
                // loop through nearby boids and sum up velocity_correction
                if num_values > 0 {
                    for value in &descendent_values {
                        //skip self if found
                        if value.entity == my_value.entity {
                            continue;
                        }
                        let diag = value.rect.max - value.rect.min;
                        let midpoint = value.rect.min + diag / 2.;
                        let distance = midpoint.distance(my_midpoint.clone());
                        let direction_away =
                            (midpoint - my_midpoint).normalize_or_zero().extend(0.);
                        velocity_correction += direction_away / (1. + 0.1 * distance.exp());
                    }
                }
                // only apply velocity_correction if not NaN and above threshold
                if velocity_correction.length_squared() > EPS {
                    velocity.0 += BOID_AVOIDANCE_FORCE * velocity_correction;
                }
            }
        }
    });
}

pub fn avoid_screen_edges(
    mut velocity_query: Query<(&mut Velocity, &Transform), With<Boid>>,
    windows: Res<Windows>,
) {
    let mut window_size = Vec2::new(0., 0.);
    if let Some(window) = windows.get_primary() {
        window_size.x = window.width();
        window_size.y = window.height();
    } else {
        return;
    }
    let left_edge_x = -window_size.x / 2.0;
    let right_edge_x = window_size.x / 2.0;
    let top_edge_y = window_size.y / 2.0;
    let bottom_edge_y = -window_size.y / 2.0;
    velocity_query.par_for_each_mut(16, |(mut velocity, transform)| {
        let loc = transform.translation;
        let distance_to_left = (loc.x - BOID_SCALE.x - left_edge_x).abs();
        let distance_to_right = (loc.x + BOID_SCALE.x - right_edge_x).abs();
        let distance_to_top = (loc.y + BOID_SCALE.x - top_edge_y).abs();
        let distance_to_bottom = (loc.y - BOID_SCALE.x - bottom_edge_y).abs();
        let force_vec = Vec2::X / (1. + 0.1 * distance_to_left.exp())
            + Vec2::NEG_X / (1. + 0.1 * distance_to_right.exp())
            + Vec2::NEG_Y / (1. + 0.1 * distance_to_top.exp())
            + Vec2::Y / (1. + 0.1 * distance_to_bottom.exp());
        // only apply velocity_correction if not NaN and above threshold
        if force_vec.length_squared() > EPS {
            velocity.0 += BOID_WALL_AVOIDANCE_FORCE * force_vec.extend(0.);
        }
    });
}
