use bevy::prelude::*;

#[derive(Component)]
pub struct Boid;

#[derive(Component, Clone)]
pub struct Kinematics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

impl Kinematics {
    pub fn integrate(&self, t: f32, v0: Vec3) -> Vec3 {
        v0 + self.velocity * t + self.acceleration * t * t / 2.
    }
}

#[derive(Component)]
pub struct Collider;

#[derive(Default)]
pub struct CollisionEvent;
