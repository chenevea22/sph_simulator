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

const PARTICLE_RADIUS: f32 = 40.;

const MAIN_BLOCK: BlockDims<f32> = BlockDims {
    base: [
        (-SIZE_X / 2.) - 100.,
        (-SIZE_Y / 2.) - 100.,
        (-SIZE_Z / 2.) - 100.,
    ],
    size: 1400.,
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
            subdivisions: 20,
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
    let particle_x_source = (-SIZE_X / 2.) + PARTICLE_RADIUS; // left side of the window
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
