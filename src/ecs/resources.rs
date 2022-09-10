use bevy::{prelude::Entity, sprite::Rect};

use crate::util::quadtree::{Locatable, Quadtree};

pub struct EntityWrapper {
    pub entity: Entity,
    pub rect: Rect,
}

impl Locatable for EntityWrapper {
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
