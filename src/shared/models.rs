use std::collections::HashMap;

use bevy::prelude::Vec3;
use transvoxel::density::ScalarField;

#[derive(PartialEq, Debug, Copy, Clone, Hash, Eq)]
pub enum Model {
    NewModel, // not sure which shape yet
}

pub fn models_map() -> HashMap<Model, Box<dyn ScalarField<f32, f32>>> {
    let mut fields: HashMap<Model, Box<dyn ScalarField<f32, f32>>> = HashMap::new();
    fields.insert(
        Model::NewModel,
        Box::new(NewModel {
            cx: 5f32,
            _cy: 5f32,
            cz: 5f32,
            r: 100f32,
        }),
    );
    return fields;
}

pub const THRESHOLD: f32 = 0.;

pub struct ParticleModel {
    pub positions: Vec<Vec3>,
}

const PARTICLE_MASS: f32 = 50.;
const SMOOTHING_LENGTH: f32 = 80.;

const NORMALIZATION_DENSITY: f32 = (315. * PARTICLE_MASS)
    / (64.
        * core::f32::consts::PI
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH);

impl ScalarField<f32, f32> for ParticleModel {
    fn get_density(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut point_density: f32 = 0.;
        for particle_position in self.positions.iter() {
            let grid_position = Vec3::new(x, y, z);

            let distance_between = grid_position - *particle_position;
            //this is the distance
            let length = distance_between.length();

            if length < SMOOTHING_LENGTH {
                let density =
                    NORMALIZATION_DENSITY * (SMOOTHING_LENGTH.powf(2.) - length.powf(2.)).powf(3.);
                point_density += density;
            }
        }
        return point_density;
    }
}

struct NewModel {
    pub cx: f32,
    pub _cy: f32,
    pub cz: f32,
    pub r: f32,
}

impl ScalarField<f32, f32> for NewModel {
    fn get_density(&self, x: f32, _y: f32, z: f32) -> f32 {
        let distance_from_center = ((x - self.cx).powi(2)
            //+ (y - self.cy) * (y - self.cy)
            + (z - self.cz).powi(2))
        .sqrt();
        let d = 1f32 - distance_from_center / self.r;
        d
    }
}
