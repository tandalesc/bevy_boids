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
const BOID_DETECTION_RADIUS: f32 = 2.;
const BOID_GROUP_APPROACH_RADIUS: f32 = 5.;
const BOID_SPEED: f32 = 100.;

pub fn apply_kinematics(mut boid_query: Query<(&Kinematics, &mut Transform), With<Boid>>) {
    boid_query.par_for_each_mut(16, |(kinematics, mut transform)| {
        // RK4
        let y0 = transform.translation;
        let h = DELTA_TIME_FIXED;
        let k1 = kinematics.velocity;
        let k2 = ((y0 + k1 * (h / 2.)) - y0) / (h / 2.);
        let k3 = ((y0 + k2 * (h / 2.)) - y0) / (h / 2.);
        let k4 = ((y0 + k3 * h) - y0) / h;

        let dy = (k1 + (2. * k2) + (2. * k3) + k4) * h / 6.;
        transform.translation += dy;
    });
}

pub fn update_quadtree(
    entity_query: Query<(Entity, &Kinematics, &Transform), With<Boid>>,
    mut quadtree: ResMut<EntityQuadtree>,
) {
    entity_query.for_each(|(entity, kinematics, transform)| {
        // quadtrees have relatively fast delete and add operations, so just run that every time
        let value = EntityWrapper::new(entity, &kinematics.velocity, transform);
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

pub fn approach_nearby_boid_groups(
    mut velocity_query: Query<(&mut Kinematics, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    velocity_query.par_for_each_mut(4, |(mut kinematics, entity, transform)| {
        let my_value = EntityWrapper::new(entity, &kinematics.velocity, transform);
        let detection_rect =
            magnify_rect(my_value.get_rect(), Vec2::ONE * BOID_GROUP_APPROACH_RADIUS);
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            if let Some(descendent_values) = node.get_all_descendant_values() {
                let num_values = descendent_values.len();
                // loop through nearby boids and sum up velocity_correction
                if num_values > 1 {
                    let mut average_velocity = Vec3::ZERO;
                    for value in &descendent_values {
                        //skip self if found
                        if value.entity == entity {
                            continue;
                        }
                        average_velocity += value.velocity;
                    }
                    average_velocity /= num_values as f32;
                    // only apply velocity_correction if not NaN and above threshold
                    if average_velocity.length_squared() > EPS {
                        let current_dir = kinematics.velocity.normalize_or_zero();
                        let force_direction = average_velocity.normalize_or_zero();
                        let new_dir = current_dir.lerp(force_direction, 0.015);
                        kinematics.velocity = new_dir * BOID_SPEED;
                    }
                }
            }
        }
    });
}

pub fn avoid_nearby_boids(
    mut velocity_query: Query<(&mut Kinematics, Entity, &Transform), With<Boid>>,
    quadtree: Res<EntityQuadtree>,
) {
    velocity_query.par_for_each_mut(4, |(mut kinematics, entity, transform)| {
        let my_value = EntityWrapper::new(entity, &kinematics.velocity, transform);
        let my_diag = my_value.rect.max - my_value.rect.min;
        let my_midpoint = my_value.rect.min + my_diag / 2.;
        let detection_rect = magnify_rect(my_value.get_rect(), Vec2::ONE * BOID_DETECTION_RADIUS);
        // find other nearby boids using quadtree lookup and calculate velocity_correction
        if let Some(node) = quadtree.query_rect(&detection_rect) {
            if let Some(descendent_values) = node.get_all_descendant_values() {
                let mut force_vec = Vec2::ZERO;
                let num_values = descendent_values.len();
                // loop through nearby boids and sum up velocity_correction
                if num_values > 1 {
                    for value in &descendent_values {
                        //skip self if found
                        if value.entity == my_value.entity {
                            continue;
                        }
                        let diag = value.rect.max - value.rect.min;
                        let midpoint = value.rect.min + diag / 2.;
                        let distance = midpoint.distance(my_midpoint.clone());
                        let direction_away = (midpoint - my_midpoint).normalize_or_zero();
                        force_vec -= direction_away
                            / (1.
                                + (1. / BOID_SCALE.length())
                                    * (distance - BOID_SCALE.length()).exp());
                    }
                    // only apply velocity_correction if not NaN and above threshold
                    if force_vec.length_squared() > EPS {
                        let current_dir = kinematics.velocity.normalize_or_zero();
                        let force_direction = force_vec.normalize_or_zero().extend(0.);
                        let new_dir = current_dir.lerp(force_direction, 0.03);
                        kinematics.velocity = new_dir * BOID_SPEED;
                    }
                }
            }
        }
    });
}

pub fn avoid_screen_edges(
    mut velocity_query: Query<(&mut Kinematics, &Transform), With<Boid>>,
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
    velocity_query.par_for_each_mut(8, |(mut kinematics, transform)| {
        let loc = transform.translation;
        let mut new_velocity = kinematics.velocity.clone();
        let mut update_velocity = false;
        // calculate distances
        let distance_to_left = (loc.x - left_edge_x).abs();
        let distance_to_right = (loc.x - right_edge_x).abs();
        let distance_to_top = (loc.y - top_edge_y).abs();
        let distance_to_bottom = (loc.y - bottom_edge_y).abs();
        // bounce if too close to screen edge
        let x_margin = BOID_SCALE.x * 2.;
        let y_margin = BOID_SCALE.y * 2.;
        if distance_to_left < x_margin || distance_to_right < x_margin {
            new_velocity.x *= -1.;
            update_velocity = true;
        }
        if distance_to_top < y_margin || distance_to_bottom < y_margin {
            new_velocity.y *= -1.;
            update_velocity = true;
        }
        // only apply velocity_correction if not NaN and above threshold
        if update_velocity {
            kinematics.velocity = new_velocity;
        }
    });
}
