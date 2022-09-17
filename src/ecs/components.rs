use bevy::prelude::*;

#[derive(Component)]
pub struct Boid;

#[derive(Component, Clone)]
pub struct Kinematics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

#[derive(Component)]
pub struct Collider;

#[derive(Default)]
pub struct CollisionEvent;
