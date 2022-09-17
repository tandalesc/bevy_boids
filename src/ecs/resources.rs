use bevy::{
    prelude::{Entity, Transform, Vec3},
    sprite::Rect,
};

use crate::util::{
    quadtree::{quadtree::Quadtree, quadtree_value::QuadtreeValue},
    rect::transform_to_rect,
};

pub struct EntityWrapper {
    pub entity: Entity,
    pub rect: Rect,
    pub velocity: Vec3,
}

impl EntityWrapper {
    pub fn new(entity: Entity, velocity: &Vec3, transform: &Transform) -> Self {
        EntityWrapper {
            entity,
            velocity: velocity.clone(),
            rect: transform_to_rect(transform),
        }
    }
}

impl QuadtreeValue for EntityWrapper {
    fn get_rect(&self) -> &Rect {
        &self.rect
    }
}

impl PartialEq for EntityWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

pub type EntityQuadtree = Quadtree<EntityWrapper>;
