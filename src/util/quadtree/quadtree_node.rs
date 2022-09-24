use std::ops::AddAssign;

use bevy::{sprite::Rect, utils::HashSet};

use crate::util::rect::{partition_rect, rect_contains_rect};

use super::{quadtree_value::QuadtreeValue, MAX_DEPTH, THRESHOLD};

pub struct QuadtreeNode<T> {
    pub rect: Rect,
    pub depth: usize,
    pub children: Vec<QuadtreeNode<T>>,
    pub values: HashSet<T>,
}

impl<T: QuadtreeValue> QuadtreeNode<T> {
    pub fn empty(rect: Rect, depth: usize) -> Self {
        QuadtreeNode {
            rect,
            depth,
            children: vec![],
            values: HashSet::new(),
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    pub fn clean_children(&mut self) {
        let mut empty_children = 0;
        for child in &mut self.children {
            if child.is_leaf() && child.values.len() == 0 {
                empty_children += 1;
            } else {
                child.clean_children();
            }
        }
        if empty_children == 4 {
            self.children.clear();
        }
    }

    // loop through self and all descendents, run aggregation function and return summed result
    pub fn aggregate_statistic<AggT: AddAssign<AggT>, AggFn: Fn(&QuadtreeNode<T>) -> AggT>(
        &self,
        agg_func: &AggFn,
    ) -> AggT {
        let mut agg_value: AggT = agg_func(self);
        for child in &self.children {
            agg_value += child.aggregate_statistic(agg_func);
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
                self.add(value);
            }
        } else {
            if self.values.len() < THRESHOLD {
                self.values.insert(value);
            } else if self.children_contain_rect(value.get_rect()) {
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
            self.children.iter().any(|child| child.contains_rect(rect))
        }
    }

    pub fn find_value(&self, value: &T) -> Option<&QuadtreeNode<T>> {
        if self.contains_value(value) {
            return Some(self);
        }
        for child in &self.children {
            if let Some(node) = child.find_value(value) {
                if node.contains_value(value) {
                    return Some(node);
                } else {
                    return node.find_value(value);
                }
            }
        }
        None
    }

    pub fn find_value_mut(&mut self, value: &T) -> Option<&mut QuadtreeNode<T>> {
        if self.contains_value(value) {
            return Some(self);
        }
        for child in &mut self.children {
            if let Some(node) = child.find_value_mut(value) {
                if node.contains_value(value) {
                    return Some(node);
                } else {
                    return node.find_value_mut(value);
                }
            }
        }
        None
    }

    pub fn get_all_descendant_values(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        if self.is_leaf() {
            Box::new(self.values.iter())
        } else {
            Box::new(
                self.values
                    .iter()
                    .chain(self.children[0].get_all_descendant_values())
                    .chain(self.children[1].get_all_descendant_values())
                    .chain(self.children[2].get_all_descendant_values())
                    .chain(self.children[3].get_all_descendant_values()),
            )
        }
    }

    pub fn delete(&mut self, value: &T) -> Option<T> {
        // clean up children if needed
        if !self.is_leaf() {
            let delete_children = self.children.iter().all(|child| child.values.is_empty());
            if delete_children {
                self.children.clear();
            }
        }
        // delete value
        self.values.take(value)
    }

    pub fn query_rect(&self, rect: &Rect) -> Option<&QuadtreeNode<T>> {
        if !self.contains_rect(rect) {
            return None;
        }
        for child in &self.children {
            if let Some(gc) = child.query_rect(rect) {
                return Some(gc);
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
        for child in &mut self.children {
            if let Some(gc) = child.query_rect_mut(rect) {
                return Some(gc);
            }
        }
        None
    }

    fn create_children(&mut self) {
        if self.children.len() > 0 {
            return;
        }
        for rect in partition_rect(&self.rect) {
            self.children.push(QuadtreeNode::empty(rect, self.depth + 1));
        }
    }

    fn distribute_values(&mut self) {
        if self.children.len() == 0 {
            return;
        }
        let values: Vec<T> = self.values.drain().collect();
        for value in values {
            match self.query_rect_mut(value.get_rect()) {
                Some(node) => {
                    node.add(value);
                }
                None => {
                    self.add(value);
                }
            }
        }
    }
}
