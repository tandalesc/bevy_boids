use bevy::prelude::*;

#[derive(Component)]
pub struct Boid;

#[derive(Component, Deref, DerefMut)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct Collider;

#[derive(Default)]
pub struct CollisionEvent;
