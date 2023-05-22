use bevy::prelude::*;
use crate::FlyCamera;

pub(crate) fn spawn_camera(commands: &mut Commands) {
    let cam_transform =
        Transform::from_xyz(0.0, 15.0, 15.0).looking_at(Vec3::new(5.0, 5.0, 5.0), Vec3::Y);
    let mut cam_bundle = commands.spawn_bundle(Camera3dBundle {
        transform: cam_transform,
        ..Default::default()
    });
    cam_bundle.insert(FlyCamera {
        enabled: true,
        mouse_motion_enabled: false,
        key_forward: KeyCode::Up,
        key_backward: KeyCode::Down,
        key_left: KeyCode::Left,
        key_right: KeyCode::Right,
        key_up: KeyCode::PageUp,
        key_down: KeyCode::PageDown,
        sensitivity: 9.0,
        ..Default::default()
    });
}