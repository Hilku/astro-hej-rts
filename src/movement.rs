use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

#[derive(Component)]
pub struct FaceMovementDirection {
    pub last_pos: Vec3,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, face_towards_movement);
    }
}

fn face_towards_movement(mut entities: Query<(&mut Transform, &mut FaceMovementDirection)>) {
    for (mut tr, mut face_dir) in entities.iter_mut() {
        if face_dir.last_pos != tr.translation {
            let diff = tr.translation - face_dir.last_pos;
            let angle = diff.y.atan2(diff.x) - FRAC_PI_2;
            tr.rotation = tr
                .rotation
                .slerp(Quat::from_axis_angle(Vec3::Z, angle), 0.1);
        }

        face_dir.last_pos = tr.translation;
    }
}
