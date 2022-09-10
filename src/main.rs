mod ecs;
mod util;

use self::ecs::run_ecs_application;

/*
    This is an exercise in building a scalable game framework with Bevy as a base.
    Following https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs
    although not necessarily in every detail.
*/

fn main() {
    run_ecs_application();
}
