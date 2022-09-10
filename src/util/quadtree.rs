use bevy::sprite::Rect;

use super::rect::{partition_rect, rect_contains_rect};

const THRESHOLD: usize = 16;
const MAX_DEPTH: usize = 8;

pub trait Locatable {
    fn get_rect(&self) -> &Rect;
}

pub struct Quadtree<T: Locatable + PartialEq> {
    pub rect: Rect,
    pub root: QuadtreeNode<T>,
}

pub struct QuadtreeNode<T> {
    pub rect: Rect,
    depth: usize,
    children: Option<Box<[QuadtreeNode<T>; 4]>>,
    pub values: Vec<T>,
}

impl<T: Locatable + PartialEq> Quadtree<T> {
    pub fn empty(size: Rect) -> Self {
        Quadtree {
            rect: size.clone(),
            root: QuadtreeNode::<T>::empty(size.clone(), 0),
        }
    }

    pub fn count_nodes(&self) -> usize {
        1 + self.root.count_children()
    }

    pub fn count_values(&self) -> usize {
        self.root.count_containing_values()
    }

    pub fn add(&mut self, value: T) {
        self.root.add(value);
    }

    pub fn delete(&mut self, value: &T) -> Option<T> {
        if let Some(node) = self.query_value_mut(value) {
            node.delete(value)
        } else {
            None
        }
    }

    pub fn query_value(&self, value: &T) -> Option<&QuadtreeNode<T>> {
        if self.root.contains_value(value) {
            Some(&self.root)
        } else {
            self.root.find_value(value)
        }
    }

    pub fn query_value_mut(&mut self, value: &T) -> Option<&mut QuadtreeNode<T>> {
        if self.root.contains_value(value) {
            Some(&mut self.root)
        } else {
            self.root.find_value_mut(value)
        }
    }

    pub fn debug(&self) {
        println!(
            "Quadtree Stats - Nodes: {} - Values: {}",
            self.count_nodes(),
            self.count_values()
        );
    }
}

impl<T: Locatable + PartialEq> QuadtreeNode<T> {
    pub fn empty(rect: Rect, depth: usize) -> Self {
        QuadtreeNode {
            rect,
            depth,
            children: None,
            values: vec![],
        }
    }

    pub fn is_leaf(&self) -> bool {
        match self.children {
            Some(_) => false,
            None => true,
        }
    }

    pub fn count_children(&self) -> usize {
        let mut count = 0;
        if let Some(children) = &self.children {
            count += 4;
            for child in children.iter() {
                count += child.count_children();
            }
        }
        return count;
    }

    pub fn count_containing_values(&self) -> usize {
        let mut count = self.values.len();
        if let Some(children) = &self.children {
            for child in children.iter() {
                count += child.count_containing_values();
            }
        }
        return count;
    }

    pub fn add(&mut self, value: T) {
        if self.is_leaf() {
            if self.depth >= MAX_DEPTH || self.values.len() < THRESHOLD {
                self.values.push(value);
            } else {
                self.create_children();
                self.distribute_values();
            }
        } else {
            if let Some(child) = self.query_rect_mut(value.get_rect()) {
                child.add(value);
            } else {
                self.values.push(value);
            }
        }
    }

    pub fn contains_rect(&self, rect: &Rect) -> bool {
        rect_contains_rect(&self.rect, rect)
    }

    pub fn contains_value(&self, value: &T) -> bool {
        self.values.contains(value)
    }

    pub fn find_value(&self, value: &T) -> Option<&QuadtreeNode<T>> {
        if !self.contains_rect(value.get_rect()) {
            return None;
        }
        if let Some(boxed_children) = &self.children {
            for child in boxed_children.iter() {
                if let Some(node) = child.find_value(value) {
                    if node.contains_value(value) {
                        return Some(node);
                    } else if node.contains_rect(value.get_rect()) {
                        return node.find_value(value);
                    }
                }
            }
        }
        None
    }

    pub fn find_value_mut(&mut self, value: &T) -> Option<&mut QuadtreeNode<T>> {
        if !self.contains_rect(value.get_rect()) {
            return None;
        }
        if let Some(boxed_children) = &mut self.children {
            for child in boxed_children.iter_mut() {
                if let Some(node) = child.find_value_mut(value) {
                    if node.contains_value(value) {
                        return Some(node);
                    } else if node.contains_rect(value.get_rect()) {
                        return node.find_value_mut(value);
                    }
                }
            }
        }
        None
    }

    pub fn delete(&mut self, value: &T) -> Option<T> {
        for value_idx in 0..self.values.len() {
            if let Some(v) = self.values.get(value_idx) {
                if v == value {
                    return Some(self.values.remove(value_idx));
                }
            }
        }
        None
    }

    pub fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        if !self.contains_rect(rect) {
            return None;
        }
        if let Some(boxed_children) = &self.children {
            for child in boxed_children.iter() {
                if child.contains_rect(rect) {
                    return Some(child);
                }
            }
        }
        None
    }

    pub fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        if !self.contains_rect(rect) {
            return None;
        }
        if let Some(boxed_children) = &mut self.children {
            for child in boxed_children.iter_mut() {
                if child.contains_rect(rect) {
                    return Some(child);
                }
            }
        }
        None
    }

    fn create_children(&mut self) {
        if let Some(_) = &self.children {
            panic!("QuadtreeNode.create_children: Attempted to create_children after they are already created.");
        }
        let child_rects = partition_rect(&self.rect);
        self.children = Some(Box::new([
            QuadtreeNode::empty(child_rects[0], self.depth + 1),
            QuadtreeNode::empty(child_rects[1], self.depth + 1),
            QuadtreeNode::empty(child_rects[2], self.depth + 1),
            QuadtreeNode::empty(child_rects[3], self.depth + 1),
        ]));
    }

    fn distribute_values(&mut self) {
        if let None = &self.children {
            panic!("QuadtreeNode.distribute_values: Attempted to distribute_values without first calling create_children.");
        }
        let values: Vec<T> = self.values.drain(..).collect();
        for value in values.into_iter() {
            if let Some(node) = self.query_rect_mut(value.get_rect()) {
                node.add(value);
            } else {
                self.add(value);
            }
        }
    }
}
