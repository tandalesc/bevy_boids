use bevy::{
    prelude::{Entity, Transform},
    sprite::Rect,
};

use crate::util::{
    quadtree::{Quadtree, QuadtreeValue},
    rect::transform_to_rect,
};

pub struct EntityWrapper {
    pub entity: Entity,
    pub rect: Rect,
}

impl EntityWrapper {
    pub fn new(entity: Entity, transform: &Transform) -> Self {
        EntityWrapper {
            entity,
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
