use bevy::prelude::*;
// use crate::Transform;

pub(crate) fn spawn_light(commands: &mut Commands) {
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0),
        point_light: PointLight {
            range: 100.0,
            radius: 250.0,
            ..Default::default()
        },
        ..Default::default()
    });
}