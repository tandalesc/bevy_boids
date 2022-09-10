# Bevy Boids
## Experiment
- Author: Shishir Tandale
- Language: Rust
- Main Crate: Bevy
- Platform: Any (Windows/MacOS/Linux/Web)
- Topic: Spatial Partitioning, Quadtrees, Boids

## Summary
This is an exploration of Bevy and spatial 
partitioning concepts. [Bevy][1] is a 
data-driven application framework for the Rust
language. [Quadtrees][2] are a data structure 
used to speed up computation of collision and
nearest-neighbor checks. [Boids][3] are a 
concept in artifical-intelligence related to 
forming complex swarming behaviors by composing
relatively simple rules.

This is a relatively difficult problem as a
naive implementation of Boids requires
expensive loops over all other Boids -- in
other words, it is an O(n^2) algorithm. Spatial
partitioning schemes are needed to speed up
the type of calculations we need. By reducing
the problem to a tree lookup, we instead get
a time complexity around O(log(n)) for
`query`, `add`, and `delete` operations.

This was possible due to [this tutorial][4]
on Quadtrees, and [this example][5] from
Bevy.

[1]: https://bevyengine.org
[2]: https://en.wikipedia.org/wiki/Quadtree
[3]: https://en.wikipedia.org/wiki/Boids
[4]: https://pvigier.github.io/2019/08/04/quadtree-collision-detection.html
[5]: https://github.com/bevyengine/bevy/blob/latest/examples/games/breakout.rs

## How to use
This is a standard Rust/Bevy application. Use
`cargo run` to run in development mode.

## Milestones
- [x] Render Boids
- [x] Implement basic kinematics
- [X] Implement initial Quadtree implementation
- [X] Implement first boid behavior
- [ ] Implement more complex behaviors
- [ ] Refine Quadtree implementation
- [ ] Final touches