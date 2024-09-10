use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct FaceMovementDirection {
    pub face_to_pos: Vec3,
}

#[derive(Component)]
pub struct Avoidance;

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, face_towards_movement);
        app.add_systems(Update, avoid_each_other);
    }
}

fn avoid_each_other(
    mut avoiders: Query<(Entity, &GlobalTransform), With<Avoidance>>,
    mut transforms: Query<&mut Transform>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
) {
    let shape = Collider::ball(20.);
    let shape_rot = 0.;
    let filter = QueryFilter::default();
    for (e, global_tr) in avoiders.iter_mut() {
        let mut avoidance_vec = Vec3::ZERO;
        let shape_pos = global_tr.translation().truncate();

        rapier_context.intersections_with_shape(
            shape_pos,
            shape_rot,
            &shape,
            filter,
            |obstacle_entity| {
                if obstacle_entity == e {
                    return true;
                }
                if let Ok([avoider_tr, obstacle_tr]) = transforms.get_many([e, obstacle_entity]) {
                    let diff_vec = avoider_tr.translation - obstacle_tr.translation;
                    avoidance_vec += diff_vec;
                }
                true
            },
        );
        if let Ok(mut avoider_tr) = transforms.get_mut(e) {
            avoider_tr.translation +=
                avoidance_vec.normalize_or_zero() * time.delta_seconds() * 24.0;
        }
    }
}

fn face_towards_movement(
    mut entities: Query<(&mut Transform, &FaceMovementDirection, &GlobalTransform)>,
) {
    for (mut tr, face_dir, global_tr) in entities.iter_mut() {
        let diff = face_dir.face_to_pos - global_tr.translation();
        let angle = diff.y.atan2(diff.x) - FRAC_PI_2;
        tr.rotation = tr
            .rotation
            .slerp(Quat::from_axis_angle(Vec3::Z, angle), 0.1);
    }
}
