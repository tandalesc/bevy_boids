use std::ops::AddAssign;

use bevy::{
    sprite::Rect,
    utils::{hashbrown::hash_set::Iter, HashSet},
};

use crate::util::rect::{partition_rect, rect_contains_rect};

use super::{quadtree_value::QuadtreeValue, MAX_DEPTH, THRESHOLD};

pub struct QuadtreeNode<T> {
    pub rect: Rect,
    pub depth: usize,
    pub children: Option<Box<[QuadtreeNode<T>; 4]>>,
    pub values: HashSet<T>,
}

impl<T: QuadtreeValue> QuadtreeNode<T> {
    pub fn empty(rect: Rect, depth: usize) -> Self {
        QuadtreeNode {
            rect,
            depth,
            children: None,
            values: HashSet::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
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
                self.values.insert(value);
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
                self.values.insert(value);
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
        if self.is_leaf() {
            false
        } else {
            self.children
                .as_ref()
                .unwrap()
                .iter()
                .any(|child| child.contains_rect(rect))
        }
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

    pub fn get_all_descendant_values(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        if self.is_leaf() {
            Box::new(self.values.iter())
        } else {
            let children = (&self.children).as_ref().unwrap();
            Box::new(
                self.values
                    .iter()
                    .chain(children[0].get_all_descendant_values())
                    .chain(children[1].get_all_descendant_values())
                    .chain(children[2].get_all_descendant_values())
                    .chain(children[3].get_all_descendant_values()),
            )
        }
    }

    pub fn delete(&mut self, value: &T) -> Option<T> {
        // clean up children if needed
        if !self.is_leaf() {
            let delete_children = self
                .children
                .as_ref()
                .unwrap()
                .iter()
                .all(|child| child.values.len() == 0);
            if delete_children {
                self.children = None;
            }
        }
        // delete value
        self.values.take(value)
    }

    pub fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
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

    pub fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
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
        let values: Vec<T> = self.values.drain().collect();
        for value in values {
            if let Some(node) = self.query_rect_mut(value.get_rect()) {
                node.add(value);
            } else {
                self.add(value);
            }
        }
    }
}
