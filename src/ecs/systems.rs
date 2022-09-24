use bevy::prelude::*;

use crate::util::{
    quadtree::{quadtree_stats::QuadtreeStats, quadtree_value::QuadtreeValue},
    rect::{magnify_rect, transform_to_rect},
};

use super::{
    components::{Boid, Kinematics},
    resources::{EntityQuadtree, EntityWrapper},
    setup::{BOID_DIAG_LENGTH, BOID_DIAG_LEN_RECIP, BOID_SCALE},
    PHYSICS_FRAME_RATE,
};

const EPS: f32 = 0.00001;
const DELTA_TIME_FIXED: f32 = 1. / PHYSICS_FRAME_RATE as f32;
const BOID_DETECTION_RADIUS: f32 = 1.5;
const BOID_GROUP_APPROACH_RADIUS: f32 = 2.;

const THREADS_SMALL: usize = 8;
const THREADS_MEDIUM: usize = 16;
const THREADS_LARGE: usize = 32;

pub fn apply_kinematics(mut boid_query: Query<(&Kinematics, &mut Transform)>) {
    boid_query.par_for_each_mut(THREADS_LARGE, |(kinematics, mut transform)| {
        transform.translation += kinematics.integrate_rk4(DELTA_TIME_FIXED);
    });
}

pub fn update_quadtree(
    entity_query: Query<(Entity, &Kinematics, &Transform), With<Boid>>,
    mut quadtree: ResMut<EntityQuadtree>,
) {
    entity_query.for_each(|(entity, kinematics, transform)| {
        let value = EntityWrapper::new(entity, &kinematics.velocity, transform);
        if let Some(node) = quadtree.query_rect_mut(value.get_rect()) {
            if !node.contains_value(&value) {
                quadtree.delete(&value);
                quadtree.add(value);
            }
        }
    });
    // QuadtreeStats::calculate(&quadtree).print();
}

pub fn approach_nearby_boid_groups(
    mut kinematics_query: Query<(&mut Kinematics, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    kinematics_query.par_for_each_mut(THREADS_MEDIUM, |(mut kinematics, entity, transform)| {
        let my_rect = transform_to_rect(transform);
        let detection_rect = magnify_rect(&my_rect, Vec2::splat(BOID_GROUP_APPROACH_RADIUS));
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            // loop through nearby boids and sum up velocity_correction
            let mut num_values = 0;
            let mut average_velocity = Vec3::ZERO;
            for value in node
                .get_all_descendant_values()
                .filter(|&v| v.entity != entity)
            {
                average_velocity += value.velocity;
                num_values += 1;
            }
            if num_values > 1 {
                average_velocity /= num_values as f32;
                // only apply correction if not NaN and above threshold
                if average_velocity.length_squared() > EPS {
                    let current_dir = kinematics.velocity.normalize_or_zero();
                    let force_direction = average_velocity.normalize_or_zero();
                    let new_dir = current_dir.lerp(force_direction, 0.015).normalize_or_zero();
                    kinematics.velocity = new_dir * kinematics.velocity.length();
                }
            }
        }
    });
}

pub fn avoid_nearby_boids(
    mut kinematics_query: Query<(&mut Kinematics, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    kinematics_query.par_for_each_mut(THREADS_MEDIUM, |(mut kinematics, entity, transform)| {
        let my_rect = transform_to_rect(transform);
        let detection_rect = magnify_rect(&my_rect, Vec2::splat(BOID_DETECTION_RADIUS));
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            // loop through nearby boids and sum up velocity_correction
            let mut force_vec = Vec2::ZERO;
            for value in node
                .get_all_descendant_values()
                .filter(|&v| v.entity != entity)
            {
                let delta_vec = my_rect.min - value.rect.min;
                let direction_away = delta_vec.normalize_or_zero();
                force_vec -= direction_away
                    / (1.
                        + BOID_DIAG_LEN_RECIP
                            * (delta_vec.length_squared() - BOID_DIAG_LENGTH).exp());
            }
            // only apply correction if not NaN and above threshold
            if force_vec.length_squared() > EPS {
                let current_dir = kinematics.velocity.normalize_or_zero();
                let force_direction = force_vec.normalize_or_zero().extend(0.);
                let new_dir = current_dir.lerp(force_direction, 0.03).normalize_or_zero();
                kinematics.velocity = new_dir * kinematics.velocity.length();
            }
        }
    });
}

pub fn avoid_screen_edges(
    mut kinematics_query: Query<(&mut Kinematics, &Transform), With<Boid>>,
    windows: Res<Windows>,
) {
    let mut window_size = Vec2::ZERO;
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
    let margin = (BOID_SCALE / 2.).extend(0.);
    kinematics_query.par_for_each_mut(THREADS_LARGE, |(mut kinematics, transform)| {
        let loc = transform.translation + kinematics.integrate(DELTA_TIME_FIXED) + margin;
        // calculate distances
        let distance_to_left = loc.x - left_edge_x - margin.x;
        let distance_to_right = right_edge_x - loc.x - margin.x;
        let distance_to_top = top_edge_y - loc.y - margin.y;
        let distance_to_bottom = loc.y - bottom_edge_y - margin.y;
        // bounce if too close to screen edge
        if distance_to_left < EPS || distance_to_right < EPS {
            kinematics.velocity.x *= -1.;
        }
        if distance_to_top < EPS || distance_to_bottom < EPS {
            kinematics.velocity.y *= -1.;
        }
    });
}

pub fn wrap_screen_edges(
    mut kinematics_query: Query<&mut Transform, With<Boid>>,
    windows: Res<Windows>,
) {
    let mut window_size = Vec2::ZERO;
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
    let margin = (BOID_SCALE / 2.).extend(0.);
    kinematics_query.par_for_each_mut(THREADS_LARGE, |mut transform| {
        let loc = transform.translation + margin;
        // calculate distances
        let distance_to_left = loc.x - left_edge_x - margin.x;
        let distance_to_right = right_edge_x - loc.x - margin.x;
        let distance_to_top = top_edge_y - loc.y - margin.y;
        let distance_to_bottom = loc.y - bottom_edge_y - margin.y;
        // wrap if too close to screen edge
        if distance_to_left < EPS {
            transform.translation.x += distance_to_right;
        }
        if distance_to_right < EPS {
            transform.translation.x -= distance_to_left;
        }
        if distance_to_top < EPS {
            transform.translation.y -= distance_to_bottom;
        }
        if distance_to_bottom < EPS {
            transform.translation.y += distance_to_top;
        }
    });
}
