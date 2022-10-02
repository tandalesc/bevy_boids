use bevy::sprite::Rect;

use super::{quadtree_node::QuadtreeNode, quadtree_value::QuadtreeValue};

pub struct Quadtree<T: QuadtreeValue> {
    pub rect: Rect,
    pub root: QuadtreeNode<T>,
}

impl<T: QuadtreeValue> Quadtree<T> {
    pub fn empty(size: Rect) -> Self {
        Quadtree {
            rect: size,
            root: QuadtreeNode::<T>::empty(size.clone(), 0),
        }
    }

    pub fn add(&mut self, value: T) {
        //only add if value is contained within our rect
        if self.root.contains_rect(value.get_rect()) {
            self.root.add(value);
        }
    }

    pub fn delete(&mut self, value: &T) -> Option<T> {
        match self.query_value_mut(value) {
            Some(node) => node.delete(value),
            None => None,
        }
    }

    pub fn query_value_mut(&mut self, value: &T) -> Option<&mut QuadtreeNode<T>> {
        self.root.find_value_mut(value)
    }

    pub fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        self.root.query_rect(rect)
    }

    pub fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        self.root.query_rect_mut(rect)
    }
}
