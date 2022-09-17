pub mod components;
pub mod resources;
pub mod setup;
pub mod systems;

use bevy::{prelude::*, sprite::Rect, time::FixedTimestep};

use self::components::CollisionEvent;
use self::resources::EntityQuadtree;
use self::setup::{setup_camera, spawn_boids};
use self::systems::{apply_kinematics, avoid_nearby_boids, avoid_screen_edges, update_quadtree, approach_nearby_boid_groups};

const QUADTREE_SIZE: Rect = Rect {
    min: Vec2::new(-1000., -1000.),
    max: Vec2::new(1000., 1000.),
};
const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);
pub const PHYSICS_FRAME_RATE: f64 = 60.;

/*
    These systems represent game logic.
*/
pub fn run_ecs_application() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(EntityQuadtree::empty(QUADTREE_SIZE))
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_boids)
        .add_event::<CollisionEvent>()
        .add_system_set(physics_system_set(PHYSICS_FRAME_RATE))
        .add_system(bevy::window::close_on_esc)
        .run();
}

/*
    All of these systems represent the physics engine, which runs at a fixed 60 fps.
*/
fn physics_system_set(physics_frame_rate: f64) -> SystemSet {
    SystemSet::new()
        .with_run_criteria(FixedTimestep::steps_per_second(physics_frame_rate))
        .with_system(apply_kinematics)
        .with_system(update_quadtree.after(apply_kinematics))
        .with_system(avoid_nearby_boids.after(update_quadtree))
        .with_system(avoid_screen_edges.after(update_quadtree))
        .with_system(approach_nearby_boid_groups.after(update_quadtree))
}
