use bevy::prelude::*;

use bevy::render::mesh::Mesh as BevyMesh;
use transvoxel::structs::*;
use transvoxel::transition_sides::*;

#[derive(Component)]
pub struct Particle {
    velocity: Vec3,
    acceleration: Vec3,
    density: f32,
    pressure: f32,
    force: Vec3,
}

const MAIN_BLOCK: BlockDims<f32> = BlockDims {
    base: [-400.0, -200.0, -200.0],
    size: 1000.0,
};

#[derive(Component)]
struct ModelMarkerComponent {}

#[path = "./shared/models.rs"]
mod models;
use models::Model;

#[path = "./shared/utils.rs"]
mod utils;

#[path = "functions/load_materials.rs"]
mod load_materials;

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
            subdivisions: 15,
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

pub fn render_mesh(
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
