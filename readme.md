# Smoothed Particle Hydrodynamics (SPH) Fluid Simulator

![Alt text](SPH_images.png)

This should be run in release mode, otherwise the frame rate will be very low, and the simulation will be unstable. 
```
cargo run --release
```
## Controls
* Add More Particles: hold left mouse button
* Zoom: Scroll wheel
* Pan: Hold scroll wheel
* Rotate: Hold right mouse button
* Drop Orion Spacecraft: Press the Spacebar
 
## Installations
* Rust
    * installation link: https://www.rust-lang.org/tools/install
    * If you have previously installed rust run "rustup update" to update rust to its newest version

## File Layout
* main.rs
    * Outlines the functions lifecycle for each frame
    * Contains functions to spawn particles on mouse clicks
    * Currently contains some functions related to the density mesh, this will be updated soon to move to a new, separate file
    * render_mesh()
        * A rendering function that clears the scene by despawning existing models and then calls the load_model function to load and render a new mesh using the provided parameters.

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

## System Architecture Diagram
![Alt text](system_architecture_image.png)

## SPH Functions Architecture Diagram
![Alt text](sph_functions_diagram.png)

## Potential Future Work
* Update SPH Functions
    * Add a diveregence-free velocity solver and constant density solver to prevent the particles from "exploding" when too many exist within the environment
    * A good resource to start with: https://www.dankoschier.de/resources/papers/BK15.pdf
    
* Create new applications
    * Ex. water flowing through a pipe, more objects to collide with in the environment, etc.

* Update water mesh visuals
    * Allow for the water mesh to be multiple colors depending on certain variables such as density or force at individual points
* Replace cube with space capsule
    * Space capsule mesh needs to react to the water particles similarly to how the cube mesh did
