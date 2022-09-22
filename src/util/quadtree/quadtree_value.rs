use std::hash::Hash;

use bevy::sprite::Rect;

pub trait QuadtreeValue: PartialEq + Eq + Hash + Clone {
    fn get_rect(&self) -> &Rect;
}
