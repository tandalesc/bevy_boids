use bevy::{prelude::*, sprite::Rect};

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
    for (current_entity, transform) in &entity_query {
        let new_min = Vec2::new(transform.translation.x, transform.translation.y);
        let new_max = new_min + Vec2::new(transform.scale.x, transform.scale.y);
        let new_rect = Rect {
            min: new_min,
            max: new_max,
        };
        if let Some(node) = quadtree.root.query_rect_smallest_mut(&new_rect) {
            // TODO: remove and readd entitywrapper to quadtree instead of modifying rect in place
            // this should update the location of the node in the data structure
            // perhaps only do this if query_rect_smallest yields something different
            for EntityWrapper { entity, rect } in node.values.iter_mut() {
                if current_entity.eq(&entity) {
                    // println!("Updating rect for entity {:?}", &entity);
                    rect.min = new_rect.min;
                    rect.max = new_rect.max;
                }
            }
        }
    }
}

pub fn update_velocity(time: Res<Time>, mut velocity_query: Query<&mut Velocity, With<Boid>>) {
    for mut velocity in &mut velocity_query {
        velocity.0 += Vec3::NEG_Y * 9.8 * time.delta_seconds();
    }
}
