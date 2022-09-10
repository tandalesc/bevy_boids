use bevy::sprite::Rect;

use super::rect::{partition_rect, rect_contains_rect};

const THRESHOLD: usize = 16;
const MAX_DEPTH: usize = 8;

pub trait Locatable {
    fn get_rect(&self) -> &Rect;
}

pub struct Quadtree<T: Locatable> {
    pub rect: Rect,
    pub root: QuadtreeNode<T>,
}

pub struct QuadtreeNode<T> {
    pub rect: Rect,
    depth: usize,
    children: Option<Box<[QuadtreeNode<T>; 4]>>,
    pub values: Vec<T>,
}

impl<T: Locatable> Quadtree<T> {
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

    pub fn debug(&self) {
        println!(
            "Quadtree Stats - Nodes: {} - Values: {}",
            self.count_nodes(),
            self.count_values()
        );
    }
}

impl<T: Locatable> QuadtreeNode<T> {
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

    pub fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        if let Some(boxed_children) = &self.children {
            for child in boxed_children.iter() {
                if child.contains_rect(rect) {
                    return Some(child);
                }
            }
        }
        None
    }

    pub fn query_rect_smallest(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        let mut pointer = self;
        while let Some(node) = pointer.query_rect(rect) {
            if node.is_leaf() {
                return Some(node);
            } else {
                pointer = node;
            }
        }
        None
    }

    fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        if let Some(boxed_children) = &mut self.children {
            for child in boxed_children.iter_mut() {
                if child.contains_rect(rect) {
                    return Some(child);
                }
            }
        }
        None
    }

    pub fn query_rect_smallest_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        let mut pointer = self;
        while let Some(node) = pointer.query_rect_mut(rect) {
            if node.is_leaf() {
                return Some(node);
            } else {
                pointer = node;
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
