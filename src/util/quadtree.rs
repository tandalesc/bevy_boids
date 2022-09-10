use std::ops::AddAssign;

use bevy::sprite::Rect;

use super::rect::{partition_rect, rect_contains_rect};

const THRESHOLD: usize = 16;
const MAX_DEPTH: usize = 8;

pub trait QuadtreeValue: PartialEq {
    fn get_rect(&self) -> &Rect;
}

pub struct Quadtree<T: QuadtreeValue> {
    pub rect: Rect,
    pub root: QuadtreeNode<T>,
}

pub struct QuadtreeNode<T> {
    pub rect: Rect,
    depth: usize,
    children: Option<Box<[QuadtreeNode<T>; 4]>>,
    pub values: Vec<T>,
}

#[derive(Debug)]
pub struct QuadtreeStats {
    pub num_nodes: usize,
    pub num_values: usize,
    pub average_depth: f32,
    pub average_num_values: f32,
}

impl QuadtreeStats {
    // calcuates common statistics about a quadtree
    pub fn calculate<T: QuadtreeValue>(quadtree: &Quadtree<T>) -> QuadtreeStats {
        // functions
        let count_children_fn: fn(&QuadtreeNode<T>) -> usize = |node| match node.children {
            None => 0,
            Some(_) => 4,
        };
        let count_values_fn: fn(&QuadtreeNode<T>) -> usize = |node| node.values.len();
        let total_depth_fn: fn(&QuadtreeNode<T>) -> f32 = |node| node.depth as f32;
        let num_nodes = quadtree.root.aggregate_statistic(&count_children_fn);
        let num_values = quadtree.root.aggregate_statistic(&count_values_fn);
        let average_depth = quadtree.root.aggregate_statistic(&total_depth_fn) / num_nodes as f32;
        let average_num_values = num_values as f32 / num_nodes as f32;
        QuadtreeStats {
            num_nodes,
            num_values,
            average_depth,
            average_num_values,
        }
    }

    pub fn print(&self) {
        println!("{:?}", self);
    }
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
        if let Some(node) = self.query_value_mut(value) {
            node.delete(value)
        } else {
            None
        }
    }

    pub fn clean_structure(&mut self) {
        self.root.clean_children();
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

    pub fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        self.root.query_rect(rect)
    }

    pub fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        self.root.query_rect_mut(rect)
    }
}

impl<T: QuadtreeValue> QuadtreeNode<T> {
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

    pub fn clean_children(&mut self) {
        let mut empty_children = 0;
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.is_leaf() && child.values.len() == 0 {
                    empty_children += 1;
                } else {
                    child.clean_children();
                }
            }
        }
        if empty_children == 4 {
            self.children = None;
        }
    }

    // loop through self and all descendents, run aggregation function and return summed result
    pub fn aggregate_statistic<AggT: AddAssign<AggT>, AggFn: Fn(&QuadtreeNode<T>) -> AggT>(
        &self,
        agg_func: &AggFn,
    ) -> AggT {
        let mut agg_value: AggT = agg_func(self);
        if let Some(children) = &self.children {
            for child in children.iter() {
                agg_value += child.aggregate_statistic(agg_func);
            }
        }
        return agg_value;
    }

    // add value to self if room, otherwise propagate to children, fall back to self if needed
    pub fn add(&mut self, value: T) {
        if self.is_leaf() {
            if self.depth >= MAX_DEPTH || self.values.len() < THRESHOLD {
                self.values.push(value);
            } else {
                self.create_children();
                self.distribute_values();
            }
        } else {
            if self.children_contain_rect(value.get_rect()) {
                if let Some(child) = self.query_rect_mut(value.get_rect()) {
                    child.add(value);
                }
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

    // helper function to determine if one or more children can hold this rect entirely
    pub fn children_contain_rect(&self, rect: &Rect) -> bool {
        if let Some(boxed_children) = &self.children {
            for child in boxed_children.iter() {
                if child.contains_rect(rect) {
                    return true;
                }
            }
        }
        false
    }

    pub fn find_value(&self, value: &T) -> Option<&QuadtreeNode<T>> {
        if self.contains_value(value) {
            return Some(self);
        }
        if let Some(boxed_children) = &self.children {
            for child in boxed_children.iter() {
                if let Some(node) = child.find_value(value) {
                    if node.contains_value(value) {
                        return Some(node);
                    } else {
                        return node.find_value(value);
                    }
                }
            }
        }
        None
    }

    pub fn find_value_mut(&mut self, value: &T) -> Option<&mut QuadtreeNode<T>> {
        if self.contains_value(value) {
            return Some(self);
        }
        if let Some(boxed_children) = &mut self.children {
            for child in boxed_children.iter_mut() {
                if let Some(node) = child.find_value_mut(value) {
                    if node.contains_value(value) {
                        return Some(node);
                    } else {
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

    fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        if !self.contains_rect(rect) {
            return None;
        }
        if let Some(boxed_children) = &self.children {
            for child in boxed_children.iter() {
                if let Some(gc) = child.query_rect(rect) {
                    return Some(gc);
                }
            }
        }
        Some(self)
    }

    fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        if !self.contains_rect(rect) {
            return None;
        }
        if !self.children_contain_rect(rect) {
            return Some(self);
        }
        if let Some(boxed_children) = self.children.as_mut() {
            for child in boxed_children.iter_mut() {
                if let Some(gc) = child.query_rect_mut(rect) {
                    return Some(gc);
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
