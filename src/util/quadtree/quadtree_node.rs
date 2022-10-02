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
            } else if let Some(child) = self.get_child_containing_rect_mut(value.get_rect()) {
                child.add(value);
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

    pub fn get_child_containing_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        self.children
            .iter_mut()
            .find(|child| child.contains_rect(rect))
    }

    pub fn find_value_mut(&mut self, value: &T) -> Option<&mut QuadtreeNode<T>> {
        if self.contains_value(value) {
            return Some(self);
        }
        self.children
            .iter_mut()
            .find_map(|c| c.find_value_mut(value))
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
        match self.children.iter().find_map(|c| c.query_rect(rect)) {
            Some(gc) => Some(gc),
            None => Some(self),
        }
    }

    pub fn query_rect_mut(&mut self, rect: &Rect) -> Option<&mut QuadtreeNode<T>> {
        if !self.contains_rect(rect) {
            return None;
        }
        if !self.children_contain_rect(rect) {
            return Some(self);
        }
        if let Some(gc) = self
            .children
            .iter_mut()
            .find_map(|c| c.query_rect_mut(rect))
        {
            return Some(gc);
        }
        None
    }

    pub fn get_all_descendant_nodes(&self) -> Box<dyn Iterator<Item = &QuadtreeNode<T>> + '_> {
        Box::new(
            self.children
                .iter()
                .filter(|c| !c.is_leaf())
                .flat_map(|c| c.get_all_descendant_nodes()),
        )
    }

    pub fn get_all_descendant_values(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        Box::new(
            self.values.iter().chain(
                self.get_all_descendant_nodes()
                    .flat_map(|c| c.values.iter()),
            ),
        )
    }

    fn create_children(&mut self) {
        if self.children.len() > 0 {
            return;
        }
        self.children.extend(
            partition_rect(&self.rect)
                .iter()
                .map(|&rect| QuadtreeNode::empty(rect, self.depth + 1)),
        );
    }

    fn distribute_values(&mut self) {
        if self.children.len() == 0 {
            return;
        }
        let values: Vec<T> = self.values.drain().collect();
        for value in values {
            if let Some(child) = self.get_child_containing_rect_mut(value.get_rect()) {
                child.add(value);
            } else {
                self.add(value);
            }
        }
    }
}
