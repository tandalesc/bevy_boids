use bevy::sprite::Rect;

pub trait QuadtreeValue: PartialEq {
    fn get_rect(&self) -> &Rect;
}