use std::f32::consts::FRAC_PI_2;

use crate::GamePhase;
use crate::MainCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct FaceMovementDirection {
    pub face_to_pos: Vec3,
}

#[derive(Component)]
pub struct Avoidance {
    pub last_frame_pos: Vec3,
    pub currently_avoiding: bool,
}

#[derive(Component)]
pub struct MoveForward {
    pub speed: f32,
}

pub struct MovementPlugin;

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, face_towards_movement);
        app.add_systems(
            Update,
            (avoid_each_other, camera_mover, move_forward).run_if(in_state(GamePhase::Playing)),
        );
        app.init_resource::<LastCursorPos>();
    }
}

fn avoid_each_other(
    mut avoiders: Query<(Entity, &GlobalTransform, &mut Avoidance)>,
    mut transforms: Query<&mut Transform>,
    rapier_context: Res<RapierContext>,
    time: Res<Time>,
) {
    let shape = Collider::ball(20.);
    let shape_rot = 0.;
    let filter = QueryFilter::default();
    for (e, global_tr, mut avoidance) in avoiders.iter_mut() {
        avoidance.currently_avoiding = false;
        if avoidance.last_frame_pos == global_tr.translation() {
            continue;
        }
        avoidance.last_frame_pos = global_tr.translation();
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
                    avoidance.currently_avoiding = true;
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

#[derive(Resource, Default)]
struct LastCursorPos(Vec2);

fn camera_mover(
    mut camera: Query<&mut Transform, With<MainCamera>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    mut last_cursor_pos: ResMut<LastCursorPos>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mut c_tr in camera.iter_mut() {
        //Check if cursor is at the window's edge - then check which edge    // Games typically only have one window (the primary window)
        let mut camera_mover_vec = Vec3::ZERO;
        let window = q_windows.single();
        if let Some(position) = window.cursor_position() {
            last_cursor_pos.0 = position;
        } else {
            //println!("Cursor is not in the game window.");
        }

        let border_amount = 2;

        if keyboard_input.pressed(KeyCode::KeyW) {
            camera_mover_vec += Vec3::new(0., 1., 0.);
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            camera_mover_vec += Vec3::new(-1., 0., 0.);
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            camera_mover_vec += Vec3::new(0., -1., 0.);
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            camera_mover_vec += Vec3::new(1., 0., 0.);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if last_cursor_pos.0.x >= (window.resolution.physical_width() - border_amount) as f32 {
            camera_mover_vec += Vec3::new(1., 0., 0.);
        } else if last_cursor_pos.0.x <= border_amount as f32 {
            camera_mover_vec += Vec3::new(-1., 0., 0.);
        }

        #[cfg(not(target_arch = "wasm32"))]
        if last_cursor_pos.0.y >= (window.resolution.physical_height() - border_amount) as f32 {
            camera_mover_vec += Vec3::new(0., -1., 0.);
        } else if last_cursor_pos.0.y <= border_amount as f32 {
            camera_mover_vec += Vec3::new(0., 1., 0.);
        }

        c_tr.translation += camera_mover_vec * time.delta_seconds() * 1000.;
    }
}

fn move_forward(time: Res<Time>, mut all_elements: Query<(&mut Transform, &MoveForward)>) {
    for (mut tr, move_forward) in all_elements.iter_mut() {
        let forward = tr.right();
        tr.translation += forward * move_forward.speed * time.delta_seconds();
    }
}
