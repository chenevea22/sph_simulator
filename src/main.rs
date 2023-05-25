use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::shape::UVSphere,
};

use bevy::prelude::*;
use bevy::render::mesh::Mesh as BevyMesh;
use transvoxel::structs::*;
use transvoxel::transition_sides::*;

#[path = "./shared/models.rs"]
mod models;
use models::Model;

#[path = "./shared/utils.rs"]
mod utils;

use rand::{thread_rng, Rng};
pub mod camera;

#[path = "functions/load_materials.rs"]
mod load_materials;

mod sph;
use sph::movement_system;
use sph::particle_collision_system;
use sph::pressure_and_density_system;
use sph::wall_collision_system;
use sph::SIZE_X;
use sph::SIZE_Y;
use sph::SIZE_Z;

mod box_functions;
use box_functions::add_mesh;
use box_functions::box_collision_system;

// mod marching_cubes;
// use marching_cubes::render_mesh;

const SPAWN_RATE: f32 = 5.; // waves of particles created per second
const SPAWN_WIDTH_RATIO: f32 = 0.25; // ratio of top area used to create particles

// const SIZE_X: f32 = 1600.;
// const SIZE_Y: f32 = 800.;
// const SIZE_Z: f32 = 400.;

const PARTICLE_RADIUS: f32 = 40.;

const MAIN_BLOCK: BlockDims<f32> = BlockDims {
    base: [
        (-SIZE_X / 2.) - 100.,
        (-SIZE_Y / 2.) - 100.,
        (-SIZE_Z / 2.) - 100.,
    ],
    size: 2000.,
};

#[derive(Component)]
pub struct BoxCollision;

#[derive(Resource)]
struct BevyCounter {
    pub count: usize,
}

#[derive(Component)]
pub struct Particle {
    velocity: Vec3,
    acceleration: Vec3,
    density: f32,
    pressure: f32,
    force: Vec3,
}

#[derive(Component)]
pub struct Body {
    velocity: Vec3,
    force: Vec3,
}

#[derive(Resource)]
struct ParticleScheduled {
    wave: usize,
}

#[derive(Component)]
struct ModelMarkerComponent {}

#[derive(Resource)]
struct ModelParams {
    pub model: Model,
    pub wireframe: bool,
    pub subdivisions: usize,
    pub show_grid: bool,
    pub with_transition: bool,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            model: Model::NewModel,
            wireframe: false,
            subdivisions: 30,
            show_grid: false,
            with_transition: false,
        }
    }
}

#[derive(Resource, Default)]
struct MaterialsResource {
    pub solid_model: Handle<StandardMaterial>,
    pub wireframe_model: Handle<StandardMaterial>,
    pub grid: Handle<StandardMaterial>,
    pub grid_dot: Handle<StandardMaterial>,
}

// enum AppEvent {
//     //LoadModel,
// }

fn render_mesh(
    // mut events: EventReader<AppEvent>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<BevyMesh>>,
    mats_cache: Res<MaterialsResource>,
    models_query: Query<(Entity, &ModelMarkerComponent)>,
    params: Res<ModelParams>,
    particle_query: Query<(&Particle, &Transform)>,
) {
    // let params = &ui_state.desired_things;
    for (entity, _) in models_query.iter() {
        commands.entity(entity).despawn();
    }
    load_model(
        &mut commands,
        &mut meshes,
        &mats_cache,
        &params,
        particle_query,
    ); // where everything happens
}

fn add_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    mats_cache: &Res<MaterialsResource>,
    model_params: &ModelParams,
    transition_sides: &TransitionSides,
) {
    let block = Block {
        dims: MAIN_BLOCK,
        subdivisions: model_params.subdivisions,
    };
    let grid_mesh = utils::grid_lines(&block, &transition_sides);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(grid_mesh),
            material: mats_cache.grid.clone(),
            ..Default::default()
        })
        .insert(ModelMarkerComponent {});
    let cube = BevyMesh::from(shape::Cube { size: 1.0 });
    let cube_handle = meshes.add(cube);
    for (x, y, z) in utils::inside_grid_points(&model_params.model, &block, &transition_sides) {
        let cell_size = MAIN_BLOCK.size / model_params.subdivisions as f32;
        let point_size = cell_size * 0.05;
        let resize = Transform::from_scale(Vec3::new(point_size, point_size, point_size));
        let rotate = Transform::from_rotation(Quat::from_euler(
            EulerRot::YXZ,
            45f32.to_radians(),
            45f32.to_radians(),
            0.0,
        ));
        let translate = Transform::from_xyz(x, y, z);
        commands
            .spawn(PbrBundle {
                mesh: cube_handle.clone(),
                material: mats_cache.grid_dot.clone(),
                transform: translate * rotate * resize,
                ..Default::default()
            })
            .insert(ModelMarkerComponent {});
    }
}

fn load_model(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    mats_cache: &Res<MaterialsResource>,
    model_params: &ModelParams,
    particle_query: Query<(&Particle, &Transform)>,
) {
    let wireframe = model_params.wireframe;
    let transition_sides = if model_params.with_transition {
        TransitionSide::LowX.into()
    } else {
        no_side()
    };
    let block = Block {
        dims: MAIN_BLOCK,
        subdivisions: model_params.subdivisions,
    };

    let mut positions: Vec<Vec3> = Vec::new();
    for (_particle, transform) in particle_query.iter() {
        positions.push(transform.translation);
    }

    let bevy_mesh = utils::mesh_for_model(positions, wireframe, &block, &transition_sides);
    let mat = if wireframe {
        mats_cache.wireframe_model.clone()
    } else {
        mats_cache.solid_model.clone()
    };
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(bevy_mesh),
            material: mat,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(ModelMarkerComponent {});
    if model_params.show_grid {
        add_grid(
            commands,
            meshes,
            mats_cache,
            model_params,
            &transition_sides,
        );
    }
}

fn main() {
    App::new()
        // bevy setup stuff
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                window_level: bevy::window::WindowLevel::AlwaysOnTop,
                ..default()
            }),

            ..default()
        }))
        /*.insert_resource(WindowDescriptor {
            title: "Particles!".to_string(),
            width: 1200.,
            height: 900.,
            present_mode: PresentMode::AutoNoVsync,
            resizable: true,
            ..default()
        })*/
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .insert_resource(BevyCounter { count: 0 })
        // camera setup
        .add_startup_system(camera::spawn_camera)
        .add_system(camera::pan_orbit_camera)
        // Particles setup
        .add_startup_system(setup)
        //.add_startup_system(initialize_octree)
        .add_system(mouse_handler)
        //.add_system(movement_system)
        //.add_system(populate_octree)
        //.add_system(pressure_and_density_system.after(populate_octree))
        .add_system(pressure_and_density_system)
        .add_system(particle_collision_system.after(pressure_and_density_system))
        .add_system(wall_collision_system.after(particle_collision_system))
        .add_system(movement_system.after(wall_collision_system))
        .add_system(counter_system)
        .add_system(add_mesh)
        .add_system(box_collision_system)
        .insert_resource(FixedTime::new_from_secs(1. / SPAWN_RATE))
        .add_system(scheduled_spawner)
        .init_resource::<ModelParams>()
        .init_resource::<MaterialsResource>()
        .add_system(render_mesh)
        .run();
}

// Scheduler is used to control the rate that particles are spawned
fn scheduled_spawner(
    mut commands: Commands,
    mut scheduled: ResMut<ParticleScheduled>,
    mut counter: ResMut<BevyCounter>,
    mut meshes: ResMut<Assets<BevyMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if scheduled.wave > 0 {
        spawn_particles(&mut commands, &mut counter, &mut meshes, &mut materials);
        scheduled.wave -= 1;
    }
}

#[derive(Component)]
struct StatsText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mats_cache: ResMut<MaterialsResource>,
) {
    warn!(include_str!("warning_string.txt"));

    // Setup text for particle count and FPS
    commands
        .spawn(
            TextBundle::from_sections([
                TextSection::new(
                    "Particle Count: ",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.0, 1.0, 1.0),
                }),
                TextSection::new(
                    "\nAverage FPS: ",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.0, 1.0, 0.0),
                    },
                ),
                TextSection::from_style(TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgb(0.0, 1.0, 1.0),
                }),
            ])
            .with_style(Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(5.0),
                    left: Val::Px(5.0),
                    ..default()
                },
                ..default()
            }),
        )
        .insert(StatsText);

    // Add particle scheduler
    commands.insert_resource(ParticleScheduled { wave: 0 });

    // add ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.1,
    });

    load_materials::load_materials(&mut materials, mats_cache);

    // add point light
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(200.0, 600.0, 1000.0),
        point_light: PointLight {
            intensity: 20000000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            range: 2000.0,
            ..default()
        },

        ..default()
    });
}

//  Handle mouse events
fn mouse_handler(
    mouse_button_input: Res<Input<MouseButton>>,
    mut scheduled: ResMut<ParticleScheduled>,
) {
    //  If the left mouse button is pressed, and a wave of particles is not scheduled
    if mouse_button_input.pressed(MouseButton::Left) & (scheduled.wave == 0) {
        // schedule a wave of particles
        scheduled.wave += 1;
    }
}

fn spawn_particles(
    commands: &mut Commands,
    counter: &mut BevyCounter,
    meshes: &mut ResMut<Assets<BevyMesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = thread_rng(); // create random number generator

    let spawn_count = (SPAWN_WIDTH_RATIO * SIZE_X / (PARTICLE_RADIUS)).floor() as usize; // how many particle to spawn
    let particle_x_source = -SIZE_X / 2.; // left side of the window
    let particle_y_source = SIZE_Y / 2.; // top of the window
    let particle_z_source = -0. * SIZE_Z / 2.;

    for count in 0..spawn_count {
        // offset each particle so they do not start on top of each other
        let particle_offset_x = (count as f32) * PARTICLE_RADIUS;
        let particle_x = particle_x_source + particle_offset_x;

        // spawn a new particle
        commands
            .spawn(PbrBundle {
                /*mesh: meshes.add(BevyMesh::from(shape::Icosphere {
                    // add the mesh
                    radius: PARTICLE_RADIUS,
                    subdivisions: 5,
                })),*/
                mesh: meshes.add(BevyMesh::from(UVSphere {
                    radius: PARTICLE_RADIUS,
                    sectors: 32,
                    stacks: 32,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgba(1., 1., 1., 1.),
                    ..default()
                }),
                transform: Transform::from_xyz(particle_x, particle_y_source, particle_z_source), // initial particle position
                ..default()
            })
            .insert(Particle {
                velocity: Vec3::new(
                    // Set initial velocity
                    0.2 * rng.gen::<f32>(), // small random velocity in the x direction, otherwise they just stack on top of each other
                    -2. * PARTICLE_RADIUS * SPAWN_RATE as f32, // downward velocity, so the particle is out of the way when the next wave spawns
                    0.2 * rng.gen::<f32>(),
                ),
                acceleration: Vec3::ZERO,
                density: 0.,
                pressure: 0.,
                force: Vec3::ZERO,
            });
        counter.count += 1;
    }
}

fn counter_system(
    diagnostics: Res<Diagnostics>,
    counter: Res<BevyCounter>,
    mut query: Query<&mut Text, With<StatsText>>,
) {
    let mut text = query.single_mut();

    if counter.is_changed() {
        text.sections[1].value = counter.count.to_string();
    }

    if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(average) = fps.average() {
            text.sections[3].value = format!("{average:.2}");
        }
    };
}

/*

/*
 *
 *
 * Octree Integration
 *
 *
 */
// Max point for octree and nn list. Should be changed to be a dynamic size later
const MAX_POINTS: usize = 1000;

#[derive(Resource, Default)]
struct NearestNeightborList {
    // list: [Particle; MAX_POINTS]   // Figure out how to satisfy the derived default trait for the list
    list: Vec<Particle>, // Playing with Vecs for now
}

fn populate_octree(
    mut octree: ResMut<Octree>,
    particle_query: Query<(&mut Particle, &Transform, Entity)>,
) {
    let mut new_octree = Octree::new(10);

    for (_particle, transform, entity) in &particle_query {
        let point = Point3D {
            x: transform.translation.x,
            y: transform.translation.y,
            z: transform.translation.z,
            entity,
        };
        new_octree.insert(point);
    }

    *octree = new_octree;
    //somehow create nearest neighbors list global variable here
}

#[derive(Clone, Copy)]
struct Point3D {
    x: f32,
    y: f32,
    z: f32,
    entity: Entity,
}

// An octree node could be either a branch (another octree) or it could be a leaf (which holds the particle data)

enum OctreeNode {
    Branch([Option<Box<OctreeNode>>; 8]),
    Leaf(Vec<Point3D>),
}

// Octree Stucture
#[derive(Resource)]
struct Octree {
    root: OctreeNode,
    max_depth: usize,
    length: f32,
    center: Vec3,
}

fn initialize_octree(mut commands: Commands) {
    commands.insert_resource(Octree::new(10));
}

impl OctreeNode {
    // Insert a new point into the octree
    fn insert(
        &mut self,
        point: Point3D,
        depth: usize,
        max_depth: usize,
        length: f32,
        center: Vec3,
    ) {
        // Checks to see if the current node is a branch node or a leaf
        match self {
            // Handles the case when the current node is a branch node
            // Perform checks and insert when the current node is a branch
            OctreeNode::Branch(children) => {
                let mut new_center = center;

                let x = point.x;
                let y = point.y;
                let z = point.z;

                let mut index = 0;

                // Checks to see which octant to put the new point into
                if x > center.x {
                    index |= 1;
                    new_center.x += length;
                } else {
                    new_center.x -= length;
                }
                if y > center.y {
                    index |= 2;
                    new_center.y += length;
                } else {
                    new_center.y -= length;
                }
                if z > center.z {
                    index |= 4;
                    new_center.z += length;
                } else {
                    new_center.z -= length;
                }

                // Check to see if there is a child node and create one if there isn't
                if children[index].is_none() {
                    //Is this where we give the entity and store it?
                    children[index] = Some(Box::new(OctreeNode::Leaf(Vec::new())));
                }

                // If there is a child node, then get the branch and insert the new point into the branch
                if let Some(mut child) = children[index].take() {
                    child.insert(point, depth + 1, max_depth, length / 2., new_center);
                    children[index] = Some(child);
                }
            }
            // Handles the case when the current node is a leaf node
            OctreeNode::Leaf(points) => {
                points.push(point);
                // If pushing the new point into the leaf node causes a size greater than 8 and max depth hasn't been hit,
                // Split the leaf into 8 smaller leaf nodes
                if points.len() > 8 && depth < max_depth {
                    let mut children = [None, None, None, None, None, None, None, None];
                    // Checks to see which octant to put the new point into
                    for point in points.drain(..) {
                        let mut new_center = center;

                        let x = point.x;
                        let y = point.y;
                        let z = point.z;

                        let mut index = 0;

                        // Checks to see which octant to put the new point into
                        if x > center.x {
                            index |= 1;
                            new_center.x += length;
                        } else {
                            new_center.x -= length;
                        }
                        if y > center.y {
                            index |= 2;
                            new_center.y += length;
                        } else {
                            new_center.y -= length;
                        }
                        if z > center.z {
                            index |= 4;
                            new_center.z += length;
                        } else {
                            new_center.z -= length;
                        }

                        // Check to see if there is a child node and create one if there isn't
                        if children[index].is_none() {
                            children[index] = Some(Box::new(OctreeNode::Leaf(Vec::new())));
                        }
                        // If there is a child node, then get the branch and insert the new point into the branch
                        if let Some(mut child) = children[index].take() {
                            child.insert(point, depth + 1, max_depth, length / 2., new_center);
                            children[index] = Some(child);
                        }
                    }
                    // Once leaf node is split into more nodes, change itself into a branch
                    *self = OctreeNode::Branch(children);
                }
            }
        }
    }
    fn get_points_by_index() {
        //return all of the points in a single leaf
        //recursive, possibly return empty list of neighbors when next would be out of bounds
    }
}

impl Octree {
    // Create a new and empty octree object
    fn new(max_depth: usize) -> Self {
        Octree {
            root: OctreeNode::Leaf(Vec::new()),
            max_depth: max_depth,
            length: 400.,
            center: Vec3::new(0., 0., 0.),
        }
    }
    fn nearest_neighbor_list(&self) {
        // Perform bitwise operation to get neighboring cells (size of particle)
        neighbors_offset_list();
        neighbors_list = Vec::new();

        //indexs is a list of byte indices for each leaf
        for index in indexs {
            //get points by index
            points = get_points_by_index(index);
            for (i, point) in points.enumerate() {
                //start_at might not be correct, but start at index i
                for p in points.start_at(i) {
                    neighbor_list.push(point, p);
                }
            }

            // Push neighboring particles into list (offseted amount, the 7 neighboring cells)
            // TODO: Handle out of bounds neighbor
            for neighbor_offset in neighbor_offset_list {
                neighbor_index = neighbor_offset + index;
                neighbor_points = get_points_by_index(neighbor_index);
                for point in points {
                    for neighbor_point in neighbor_points {
                        neighbor_list.push((point, neighbor_point));
                    }
                }
            }
        }
        return neighbor_list;
    }
    // Insert a point into the current octree
    fn insert(&mut self, point: Point3D) {
        self.root
            .insert(point, 0, self.max_depth, self.length, self.center);
    }

    // Traverse the tree to print out all the points in it
    fn traverse(&self, node: &OctreeNode) {
        // Check to see if current node is a branch or leaf
        match node {
            // For branches, recursively call the traverse function on all children that isn't none
            OctreeNode::Branch(children) => {
                for child in children.iter() {
                    if let Some(child) = child {
                        self.traverse(child);
                    }
                }
            }
            // For leafs, iterate through all the points in the leaf and print out the coordinate
            OctreeNode::Leaf(points) => {
                for point in points.iter() {
                    println!("Point: ({}, {}, {})", point.x, point.y, point.z);
                }
            }
        }
    }

    fn search(&self, center: Point3D, radius: f32) -> Vec<Point3D> {
        let mut points = Vec::new();
        self.search_recursive(&self.root, &center, radius, &mut points);
        return points;
    }

    // Helper function for recursively searching the tree
    fn search_recursive(
        &self,
        node: &OctreeNode,
        center: &Point3D,
        radius: f32,
        points: &mut Vec<Point3D>,
    ) {
        match node {
            // For branches, recursively call the search function on all children that isn't none
            OctreeNode::Branch(children) => {
                for child in children.iter() {
                    if let Some(child) = child {
                        self.search_recursive(child, center, radius, points);
                    }
                }
            }
            // For leafs, iterate through all the points in the leaf and check if they are within the given radius from the center point
            OctreeNode::Leaf(child_points) => {
                for point in child_points.iter() {
                    let dist_x = center.x - point.x;
                    let dist_y = center.y - point.y;
                    let dist_z = center.z - point.z;
                    let dist_sq = dist_x.powi(2) + dist_y.powi(2) + dist_z.powi(2);
                    if dist_sq <= radius.powi(2) {
                        // Add point to the list of found points
                        points.push(*point);
                    }
                }
            }
        }
    }
}

*/
