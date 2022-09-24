use bevy::prelude::*;

#[derive(Component)]
pub struct Boid;

#[derive(Component, Clone)]
pub struct Kinematics {
    pub velocity: Vec3,
    pub acceleration: Vec3,
}

impl Kinematics {
    pub fn integrate(&self, t: f32) -> Vec3 {
        self.velocity * t + self.acceleration * t * t / 2.
    }

    pub fn integrate_rk4(&self, h: f32) -> Vec3 {
        let v0 = self.velocity;
        let k1 = self.integrate(0.) + v0;
        let k2 = self.integrate(h / 2.) + (v0 + k1 / 2.);
        let k3 = self.integrate(h / 2.) + (v0 + k2 / 2.);
        let k4 = self.integrate(h) + (v0 + k3);
        h * (k1 + (2. * k2) + (2. * k3) + k4) / 6.
    }
}

#[derive(Component)]
pub struct Collider;

#[derive(Default)]
pub struct CollisionEvent;
