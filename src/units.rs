use crate::movement::FaceMovementDirection;
use crate::selection::{CurrentlySelected, Selectable};
use crate::MainCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_units); //Temp
        app.add_systems(Update, (command_units, move_units));
    }
}

fn spawn_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for i in 0..5 {
        cmd.spawn(SpriteBundle {
            texture: asset_server.load("units/ship_basic.png"),
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 0., 0.)),
            ..Default::default()
        })
        .insert(Collider::cuboid(25.0, 25.0))
        .insert(Sensor)
        .insert(Selectable)
        .insert(FaceMovementDirection {
            last_pos: Vec3::ZERO,
        })
        .insert(Velocity(150.));
    }
}
#[derive(Component)]
pub enum UnitCommand {
    MoveToPos(Vec3),
    MoveToEntity(Entity),
}

#[derive(Component)]
struct Velocity(f32);

fn command_units(
    buttons: Res<ButtonInput<MouseButton>>,
    currently_selected: Res<CurrentlySelected>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    rapier_context: Res<RapierContext>,
    mut cmd: Commands,
) {
    if buttons.just_pressed(MouseButton::Right) {
        let (camera, camera_transform) = q_camera.single();
        let window = q_window.single();
        let mut click_pos = Vec2::new(0., 0.);
        if let Some(world_position) = window
            .cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            click_pos = world_position;
        }

        let filter = QueryFilter::default();
        let mut clicked_units = Vec::new();
        rapier_context.intersections_with_point(click_pos, filter, |entity| {
            clicked_units.push(entity);
            true
        });

        let mut index = -(currently_selected.ent.len() as f32 / 2.) as i32;
        for e in currently_selected.ent.iter() {
            let mut moving_to_unit = false;
            for clicked_e in clicked_units.iter() {
                if e != clicked_e {
                    moving_to_unit = true;
                    cmd.entity(*e).insert(UnitCommand::MoveToEntity(*clicked_e));
                }
            }

            if !moving_to_unit {
                cmd.entity(*e).insert(UnitCommand::MoveToPos(
                    click_pos.extend(0.)
                        + Vec3::new(80., 0., 0.) * index as f32
                        + Vec3::new(0., -40., 0.) * index.abs() as f32, //NAIVE formation
                ));
            }
            index += 1;
        }
    }
}

fn move_units(time: Res<Time>, mut units: Query<(&mut Transform, &Velocity, &UnitCommand)>) {
    for (mut tr, vel, command) in units.iter_mut() {
        match command {
            UnitCommand::MoveToPos(pos) => {
                let dif_vec = *pos - tr.translation;
                if dif_vec.length() > 1. {
                    tr.translation += dif_vec.normalize() * vel.0 * time.delta_seconds();
                }
            }
            _ => {
                println!("Assigned command i cannot yet do!");
            }
        }
    }
}
