use bevy::prelude::*;

use crate::{Body, BoxCollision, Particle};

const GRAVITY: f32 = -200.;

const BODY_MASS: f32 = 0.01;
const BODY_SIZE: f32 = 150.;

const PARTICLE_DAMPING: f32 = 10.;

const COEF_REST: f32 = 0.828;

pub const SIZE_X: f32 = 800.;
pub const SIZE_Y: f32 = 400.;
pub const SIZE_Z: f32 = 400.;

const PARTICLE_MASS: f32 = 50.;
const ISOTROPIC_EXPONENT: f32 = 300000.;
const BASE_DENSITY: f32 = 0.00025;
const SMOOTHING_LENGTH: f32 = 80.;
const DYNAMIC_VISCOSITY: f32 = 2.0;

//const PI: f32 = std::f32::consts::PI;
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

const NORMALIZATION_PRESSURE_FORCE: f32 = (-45. * PARTICLE_MASS)
    / (core::f32::consts::PI
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH);

const NORMALIZATION_VISCOUS_FORCE: f32 = (45. * DYNAMIC_VISCOSITY * PARTICLE_MASS)
    / (core::f32::consts::PI
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH
        * SMOOTHING_LENGTH);

// numerical integration of particle positions
pub fn movement_system(
    time: Res<Time>,
    mut particle_query: Query<(&mut Particle, &mut Transform), Without<BoxCollision>>,
    mut body_query: Query<(&mut Body, &mut Transform), With<BoxCollision>>,
) {
    //println!("start of movement system");

    let dt = time.delta_seconds();
    for (mut particle, mut transform) in &mut particle_query {
        let force: Vec3 = particle.force;
        let density: f32 = particle.density;
        transform.translation += dt * particle.velocity;
        particle.velocity += dt * (force / density + Vec3::new(0.0, GRAVITY, 0.0));

        particle.density = 0.;
        particle.pressure = 0.;
        particle.force = Vec3::ZERO;
    }

    if !body_query.is_empty() {
        let (mut body, mut body_transform) = body_query.get_single_mut().unwrap();
        let force: Vec3 = body.force;
        body_transform.translation += dt * body.velocity;
        body.velocity += dt * (force / BODY_MASS + Vec3::new(0.0, GRAVITY, 0.0));
        body.force = Vec3::ZERO;
    }
}

pub fn pressure_and_density_system(mut particle_query: Query<(&mut Particle, &Transform)>) {
    let mut combinations = particle_query.iter_combinations_mut();
    while let Some([(mut particle_0, transform_0), (mut particle_1, transform_1)]) =
        combinations.fetch_next()
    {
        let distance_between = transform_1.translation - transform_0.translation;
        //this is the distance
        let length = distance_between.length();
        if length < SMOOTHING_LENGTH {
            let density =
                NORMALIZATION_DENSITY * (SMOOTHING_LENGTH.powf(2.) - length.powf(2.)).powf(3.);
            particle_0.density += density;
            particle_1.density += density;
        }
    }

    for (mut particle, _transform) in &mut particle_query.iter_mut() {
        let own_density: f32 = NORMALIZATION_DENSITY * SMOOTHING_LENGTH.powf(2.).powf(3.);
        particle.density += own_density;
        particle.pressure = ISOTROPIC_EXPONENT * (particle.density - BASE_DENSITY);
    }
}

pub fn wall_collision_system(
    mut particle_query: Query<(&mut Particle, &Transform)>,
    mut body_query: Query<(&mut Body, &Transform)>,
) {
    let half_width = SIZE_X * 0.5;
    let half_height = SIZE_Y * 0.5;
    let half_length = SIZE_Z * 0.5;

    if !body_query.is_empty() {
        let (mut body, body_transform) = body_query.get_single_mut().unwrap();

        if body_transform.translation.x < -(half_width - (BODY_SIZE / 2.)) {
            body.velocity.x = 1.;
        }

        if body_transform.translation.x > (half_width - (BODY_SIZE / 2.)) {
            body.velocity.x = -1.;
        }

        if body_transform.translation.y < -(half_height - (BODY_SIZE / 2.)) {
            body.velocity.y = 1.;
        }

        if body_transform.translation.z < -(half_length - (BODY_SIZE / 2.)) {
            body.velocity.z = 1.;
        }

        if body_transform.translation.z > (half_length - (BODY_SIZE / 2.)) {
            body.velocity.z = -1.;
        }
    }

    for (mut particle, transform) in &mut particle_query {
        let x_pos = transform.translation.x;
        let y_pos = transform.translation.y;
        let z_pos = transform.translation.z;

        let mut out_of_bounds = false;

        if x_pos > half_width {
            particle.velocity.x = -particle.velocity.x.abs() * COEF_REST;
            out_of_bounds = true;
        }

        if x_pos < -half_width {
            particle.velocity.x = particle.velocity.x.abs() * COEF_REST;
            out_of_bounds = true;
        }

        if y_pos > half_height {
            particle.velocity.y = -particle.velocity.y.abs() * COEF_REST;
            out_of_bounds = true;
        }

        if y_pos < -half_height {
            particle.velocity.y = particle.velocity.y.abs() * COEF_REST;
            out_of_bounds = true;
        }

        if z_pos > half_length {
            particle.velocity.z = -particle.velocity.z.abs() * COEF_REST;
            out_of_bounds = true;
        }

        if z_pos < -half_length {
            particle.velocity.z = particle.velocity.z.abs() * COEF_REST;
            out_of_bounds = true;
        }

        if out_of_bounds {
            let dv = -particle.velocity * PARTICLE_DAMPING / PARTICLE_MASS;
            particle.acceleration += dv;
        }
    }
}

pub fn particle_collision_system(mut particle_query: Query<(&mut Particle, &Transform)>) {
    let mut combinations = particle_query.iter_combinations_mut();

    while let Some([(mut particle_0, transform_0), (mut particle_1, transform_1)]) =
        combinations.fetch_next()
    {
        let distance_between = transform_1.translation - transform_0.translation;
        //this is the distance
        let length = distance_between.length();
        let unit_vector = distance_between / length;

        let p0_pressure: f32 = particle_0.pressure;
        let p1_pressure: f32 = particle_1.pressure;

        if length < SMOOTHING_LENGTH {
            let density_both = particle_0.density + particle_1.density;

            //Pressure Force
            let pressure_force = NORMALIZATION_PRESSURE_FORCE
                * ((p1_pressure + p0_pressure) / density_both)
                * ((SMOOTHING_LENGTH - length).powf(2.));

            particle_0.force += pressure_force * unit_vector;
            particle_1.force -= pressure_force * unit_vector;

            let p0_velocity: Vec3 = particle_0.velocity;
            let p1_velocity: Vec3 = particle_1.velocity;

            //Viscous Force
            let viscous_force = NORMALIZATION_VISCOUS_FORCE
                * ((p1_velocity - p0_velocity) / density_both)
                * (SMOOTHING_LENGTH - length);

            particle_0.force += viscous_force;
            particle_1.force -= viscous_force;
        }
    }
}
