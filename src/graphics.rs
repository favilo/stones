use std::f32::consts::{FRAC_PI_2, PI};

use avian3d::prelude::PhysicsPickable;
use bevy::prelude::*;

pub(crate) fn setup_graphics(mut commands: Commands, cameras: Query<Entity, With<Camera>>) {
    if cameras.iter().count() > 0 {
        return;
    }

    commands.spawn((
        Name::new("Primary Camera"),
        Camera3d::default(),
        // Camera {
        //     viewport: None,
        //     order: 0,
        //     is_active: true,
        //     target: RenderTarget::Window(WindowRef::Primary),
        //     hdr: false,
        //     sub_camera_view: None,
        //     ..Default::default()
        // },
        PhysicsPickable,
        Transform::from_xyz(0.0, 0.45, 0.45).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Name::new("Directional Light"),
        DirectionalLight {
            illuminance: 5_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, PI / 5.0, -FRAC_PI_2)),
    ));
}
