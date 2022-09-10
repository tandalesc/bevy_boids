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

pub type EntityQuadtree = Quadtree<EntityWrapper>;
