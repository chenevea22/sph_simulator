use crate::MaterialsResource;
use bevy::prelude::*;

pub(crate) fn load_materials(
    materials: &mut ResMut<Assets<StandardMaterial>>,
    mut mats_cache: ResMut<MaterialsResource>,
) {
    mats_cache.solid_model = materials.add(Color::rgb(0., 0., 1.).into());
    mats_cache.wireframe_model = materials.add(StandardMaterial {
        emissive: Color::WHITE,
        unlit: true,
        ..Default::default()
    });
    mats_cache.grid = materials.add(StandardMaterial {
        base_color: Color::BLACK,
        emissive: Color::rgb(0.6, 0.6, 0.6),
        perceptual_roughness: 1.0,
        metallic: 0.0,
        reflectance: 0.0,
        ..Default::default()
    });
    mats_cache.grid_dot = mats_cache.grid.clone();
}
