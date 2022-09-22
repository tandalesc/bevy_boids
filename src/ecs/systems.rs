use bevy::prelude::*;

use crate::util::{
    quadtree::{quadtree_stats::QuadtreeStats, quadtree_value::QuadtreeValue},
    rect::magnify_rect,
};

use super::{
    components::{Boid, Kinematics},
    resources::{EntityQuadtree, EntityWrapper},
    setup::BOID_SCALE,
    PHYSICS_FRAME_RATE,
};

const EPS: f32 = 0.00001;
const DELTA_TIME_FIXED: f32 = 1. / PHYSICS_FRAME_RATE as f32;
const BOID_DETECTION_RADIUS: f32 = 1.5;
const BOID_GROUP_APPROACH_RADIUS: f32 = 2.;
pub const BOID_SPEED: f32 = 100.;

const THREADS_SMALL: usize = 8;
const THREADS_MEDIUM: usize = 16;
const THREADS_LARGE: usize = 32;

pub fn apply_kinematics(mut boid_query: Query<(&Kinematics, &mut Transform), With<Boid>>) {
    let h = DELTA_TIME_FIXED;
    boid_query.par_for_each_mut(THREADS_MEDIUM, |(kinematics, mut transform)| {
        let v0 = kinematics.velocity;
        let k1 = kinematics.integrate(0.) + v0;
        let k2 = kinematics.integrate(h / 2.) + (v0 + k1 / 2.);
        let k3 = kinematics.integrate(h / 2.) + (v0 + k2 / 2.);
        let k4 = kinematics.integrate(h) + (v0 + k3);
        let dy = h * (k1 + (2. * k2) + (2. * k3) + k4) / 6.;
        transform.translation += dy;
    });
}

pub fn update_quadtree(
    entity_query: Query<(Entity, &Kinematics, &Transform), With<Boid>>,
    mut quadtree: ResMut<EntityQuadtree>,
) {
    entity_query.for_each(|(entity, kinematics, transform)| {
        let value = EntityWrapper::new(entity, &kinematics.velocity, transform);
        if let Some(node) = quadtree.query_rect(value.get_rect()) {
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
        let my_value = EntityWrapper::new(entity, &kinematics.velocity, transform);
        let detection_rect =
            magnify_rect(my_value.get_rect(), Vec2::ONE * BOID_GROUP_APPROACH_RADIUS);
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            let mut num_values = 0;
            // loop through nearby boids and sum up velocity_correction
            let mut average_velocity = Vec3::ZERO;
            for value in node.get_all_descendant_values() {
                //skip self if found
                if value.entity != entity {
                    average_velocity += value.velocity;
                    num_values += 1;
                }
            }
            if num_values > 1 {
                average_velocity /= num_values as f32;
                // only apply correction if not NaN and above threshold
                if average_velocity.length_squared() > EPS {
                    let current_dir = kinematics.velocity.normalize_or_zero();
                    let force_direction = average_velocity.normalize_or_zero();
                    let new_dir = current_dir.lerp(force_direction, 0.015);
                    kinematics.velocity = new_dir * BOID_SPEED;
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
        let my_value = EntityWrapper::new(entity, &kinematics.velocity, transform);
        let my_diag = my_value.rect.max - my_value.rect.min;
        let my_midpoint = my_value.rect.min + my_diag / 2.;
        let detection_rect = magnify_rect(my_value.get_rect(), Vec2::ONE * BOID_DETECTION_RADIUS);
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            // loop through nearby boids and sum up velocity_correction
            let mut force_vec = Vec2::ZERO;
            for value in node.get_all_descendant_values() {
                //skip self if found
                if value.entity == my_value.entity {
                    continue;
                }
                let diag = value.rect.max - value.rect.min;
                let midpoint = value.rect.min + diag / 2.;
                let distance = midpoint.distance(my_midpoint.clone());
                let direction_away = (midpoint - my_midpoint).normalize_or_zero();
                force_vec -= direction_away
                    / (1. + (1. / BOID_SCALE.length()) * (distance - BOID_SCALE.length()).exp());
            }
            // only apply correction if not NaN and above threshold
            if force_vec.length_squared() > EPS {
                let current_dir = kinematics.velocity.normalize_or_zero();
                let force_direction = force_vec.normalize_or_zero().extend(0.);
                let new_dir = current_dir.lerp(force_direction, 0.03);
                kinematics.velocity = new_dir * BOID_SPEED;
            }
        }
    });
}

pub fn avoid_screen_edges(
    mut kinematics_query: Query<(&mut Kinematics, &Transform), With<Boid>>,
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
    kinematics_query.par_for_each_mut(THREADS_LARGE, |(mut kinematics, transform)| {
        let margin = BOID_SCALE / 2.;
        let loc = transform.translation + margin.extend(0.);
        // calculate distances
        let distance_to_left = loc.x - left_edge_x - margin.x;
        let distance_to_right = right_edge_x - loc.x - margin.x;
        let distance_to_top = top_edge_y - loc.y - margin.y;
        let distance_to_bottom = loc.y - bottom_edge_y - margin.y;
        // bounce if too close to screen edge
        let mut update_velocity = false;
        let mut new_velocity = kinematics.velocity.clone();
        if distance_to_left < EPS || distance_to_right < EPS {
            new_velocity.x *= -1.;
            update_velocity = true;
        }
        if distance_to_top < EPS || distance_to_bottom < EPS {
            new_velocity.y *= -1.;
            update_velocity = true;
        }
        // only apply velocity if updated
        if update_velocity {
            kinematics.velocity = new_velocity;
        }
    });
}

pub fn wrap_screen_edges(
    mut kinematics_query: Query<&mut Transform, With<Boid>>,
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
    kinematics_query.par_for_each_mut(THREADS_LARGE, |mut transform| {
        let margin = BOID_SCALE / 2.;
        let loc = transform.translation + margin.extend(0.);
        // calculate distances
        let distance_to_left = loc.x - left_edge_x - margin.x;
        let distance_to_right = right_edge_x - loc.x - margin.x;
        let distance_to_top = top_edge_y - loc.y - margin.y;
        let distance_to_bottom = loc.y - bottom_edge_y - margin.y;
        // wrap if too close to screen edge
        if distance_to_left < EPS || distance_to_right < EPS {
            transform.translation.x *= -1.;
        }
        if distance_to_top < EPS || distance_to_bottom < EPS {
            transform.translation.y *= -1.;
        }
    });
}
