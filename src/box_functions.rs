use bevy::prelude::*;
use bevy::render::mesh::Mesh as BevyMesh;

use bevy_mod_raycast::{ray_intersection_over_mesh, Backfaces, Ray3d};

use crate::{Body, BoxCollision, Particle};

const PARTICLE_STIFFNESS: f32 = 0.04;

//const BODY_SIZE: f32 = 150.;

static mut ORION_CAPSULE_SPAWNED: bool = false;

pub fn box_collision_system(
    meshes: Res<Assets<BevyMesh>>,
    mut particle_query: Query<(&mut Particle, &Transform)>,
    collision_query: Query<(&Handle<BevyMesh>, &mut BoxCollision, &Transform)>,
    mut body_query: Query<(&mut Body, &Transform)>,
    _input: Res<Input<KeyCode>>,
) {
    if !body_query.is_empty() {
        let (mut body, _body_transform) = body_query.get_single_mut().unwrap();

        for (mesh_handle, _box_collsion, box_transform) in &collision_query {
            for (mut particle, particle_transform) in &mut particle_query {
                if let Some(mesh) = meshes.get(mesh_handle) {
                    let mesh_to_world = box_transform.compute_matrix();
                    let from = box_transform.translation;
                    let to = particle_transform.translation;
                    let particle_vec = to - from;
                    let particle_length = particle_vec.length();
                    let ray_direction = (to - from).normalize();
                    let ray = Ray3d::new(from, ray_direction);

                    if let Some(intersection) =
                        ray_intersection_over_mesh(mesh, &mesh_to_world, &ray, Backfaces::Include)
                    {
                        // There was an intersection, check if it is before the cursor
                        // on the ray
                        let hit_distance = intersection.distance() + 50.0;
                        let deflection = hit_distance - particle_length;
                        if deflection > 0.0 {
                            //println!("Hit");
                            let force = PARTICLE_STIFFNESS * deflection * ray_direction;
                            particle.force += force;
                            body.force -= force;
                        }
                    }
                }
            }
        }
    }
}

pub fn add_mesh(
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<BevyMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::Space) && unsafe { ORION_CAPSULE_SPAWNED } == false {
        unsafe { ORION_CAPSULE_SPAWNED = true };
        let id = commands
            .spawn(PbrBundle {
                mesh: meshes.add(BevyMesh::from(shape::UVSphere {
                    radius: 125.,
                    sectors: 10,
                    stacks: 10,
                })),
                material: materials.add(Color::rgba(1.0, 0.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(-300.0, 10000.0, 0.0),
                ..Default::default()
            })
            .insert(BoxCollision)
            .insert(Body {
                velocity: Vec3::new(
                    // Set initial velocity
                    0., -1000., 0.,
                ),
                force: Vec3::ZERO,
            })
            .id();
        let my_gltf = ass.load("space_capsule.glb#Scene0");
        let mut capsule = commands.spawn(SceneBundle {
            scene: my_gltf,
            transform: Transform {
                translation: Vec3::new(0., -100., 0.),
                scale: Vec3::new(50., 50., 50.),
                ..Default::default()
            },
            ..Default::default()
        });
        capsule.set_parent(id);
    }
}
