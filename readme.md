# Particle test system 
This should be run in release mode, otherwise the frame rate will be very low, and the simulation will be unstable. 
```
cargo run --release
```
## Controls
* Add More Particles: hold left mouse button
* Zoom: scroll wheel
* Pan: Hold scroll wheel
* Rotate: Hold right mouse button
 
## Installations
* Rust
    * installation link: https://www.rust-lang.org/tools/install

## File Layout
* main.rs
    * Outlines the functions lifecycle for each frame
    * Contains functions to spawn particles on mouse clicks
    * Currently contains some functions related to the density mesh, this will be updated soon to move to a new, separate file

* sph.rs
    * Contains all of the functions needed for particle movement and interactions:
    * movement_system()
        * Loops through all of the particles in the simulation and adjusts each particle's position based off it's currently calculated velocity, density, pressure, and force
    * pressure_and_density_system()
        * Calculates the pressure and density of each particle based off how close it is to other particles
    * wall_collision_system()
        * Keeps all particles contained within the specified environment
        * Adjusts a particle's and/or "Orion Capsule's" velocity by inverting its sign when it reaches an x, y, or z bound
    * particle_collision_system()
        * Computes each particle's pressure and viscous force by looping through all combinations of particles and checking when particles are within range of collision

* box_functions.rs (soon to be changed to orion_capsule.rs)
    * add_mesh()
        * Creates the box mesh that currently represents the Orion Capsule
        * Establishes the box's initial dimensions, position, and velocity
    * box_collision_system()
        * Checks if a particle collides with the box and if so, a force (equal and opposite) is calculated and applied to both the box and the particle(s) that hit it

* camera.rs
    * Given Bevy functions that allow the camera to pan around the scene

* functions
    * load_materials.rs
        * Creates the mesh that goes over the particles to give the water-y look
    * spawn_camera.rs and spawn_light.rs are no longer necessary

* shared
    * models.rs
        * Contains the functions used to move the model based on the density of a given space in the environment
        * Basically, it keeps the mesh attached to the particles
    * utils.rs
        * Functions to keep track of points in the water mesh
    * flycam.rs is no longer necessary

## Some Things to improve:
* Make 3d: Current implementation is only 2d, although it should not be too difficult to extend to 3d.
* Particle interactions: Currently the particles are basically bouncy balls with a bit of damping. Should implement actual SPH algorithm.
* Visualization:
    * Sphere implementation could be improved
    * Marching Cubes
* Nearest neighbors: Currently checks every particle against every other. Very Slow! Create Octree?
* Stability: When there are a large number of particles, the simulation can become unstable.
* Performance: Make it run faster. Run on GPU. Better algorithms?
* Time Step Control: Currently the sim runs as fast as possible and animates in real time. We need to be able to specify the time step, which could lead to slower real time simulation.  
* When too many particles are spawned, they start to disappear, and simulation does not behave correctly. 
    * Maybe a mesh limit?
    * Numerical Stability?