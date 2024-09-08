use std::f32::consts::E;
use std::process::Command;

use crate::movement::FaceMovementDirection;
use crate::selection::{CurrentlySelected, Selectable, Team};
use crate::MainCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_units, spawn_enemy_units, spawn_command_highlighters),
        ); //Temp
        app.add_systems(Update, (command_units, move_units));
        app.add_systems(PostUpdate, display_command_of_selection);
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
        .insert(Velocity(150.))
        .insert(UnitCommandList {
            commands: Vec::new(),
        })
        .insert(Team(0));
    }
}
fn spawn_enemy_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for i in 0..5 {
        cmd.spawn(SpriteBundle {
            texture: asset_server.load("units/ship_basic.png"),
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 300., 0.)),
            sprite: Sprite {
                color: Color::srgb(1., 0.5, 0.5),
                ..default()
            },
            ..Default::default()
        })
        .insert(Collider::cuboid(25.0, 25.0))
        .insert(Sensor)
        .insert(Selectable)
        .insert(FaceMovementDirection {
            last_pos: Vec3::ZERO,
        })
        .insert(UnitCommandList {
            commands: Vec::new(),
        })
        .insert(Velocity(150.))
        .insert(Team(1));
    }
}
#[derive(Component, Clone, Copy)]
pub enum UnitCommand {
    MoveToPos(Vec3),
    MoveToEntity(Entity),
    Completed,
}

#[derive(Component)]
pub struct UnitCommandList {
    commands: Vec<UnitCommand>,
}

#[derive(Component)]
struct Velocity(f32);

fn command_units(
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    currently_selected: Res<CurrentlySelected>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    rapier_context: Res<RapierContext>,
    mut q_unit_command_list: Query<&mut UnitCommandList>,
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
                    // cmd.entity(*e).insert(UnitCommand::MoveToEntity(*clicked_e));
                }
            }

            if !moving_to_unit {
                if let Ok(mut unit_command_list) = q_unit_command_list.get_mut(*e) {
                    if !keyboard_input.pressed(KeyCode::ShiftLeft) {
                        unit_command_list.commands = Vec::new();
                    }
                    unit_command_list.commands.push(UnitCommand::MoveToPos(
                        click_pos.extend(0.)
                            + Vec3::new(80., 0., 0.) * index as f32
                            + Vec3::new(0., -40., 0.) * index.abs() as f32, //NAIVE formation
                    ))
                }
            }
            index += 1;
        }
    }
}

fn move_units(
    time: Res<Time>,
    mut units: Query<(&mut Transform, &Velocity, &mut UnitCommandList)>,
) {
    for (mut tr, vel, mut command_list) in units.iter_mut() {
        if command_list.commands.len() > 0 {
            let command = &mut command_list.commands[0];
            match command {
                UnitCommand::MoveToPos(pos) => {
                    let dif_vec = *pos - tr.translation;
                    if dif_vec.length() > 2. {
                        tr.translation += dif_vec.normalize() * vel.0 * time.delta_seconds();
                    } else {
                        *command = UnitCommand::Completed;
                    }
                }
                UnitCommand::Completed => {
                    command_list.commands.remove(0);
                }
                _ => {
                    println!("Assigned command i cannot yet do!");
                }
            }
        }
    }
}

fn display_command_of_selection(
    currently_selected: Res<CurrentlySelected>,
    q_unit_command_list: Query<&UnitCommandList>,
    mut command_highlighters: Query<&mut Transform, With<CommandHighlighter>>,
) {
    for mut tr in command_highlighters.iter_mut() {
        tr.translation = Vec3::new(1000000., 9999999., -1.);
    }

    let mut all_highlighters = command_highlighters.iter_mut();
    for selected in currently_selected.ent.iter() {
        if let Ok(command) = q_unit_command_list.get(*selected) {
            for c in &command.commands {
                match c {
                    UnitCommand::MoveToPos(pos) => {
                        if let Some(mut highlighter_tr) = all_highlighters.next() {
                            highlighter_tr.translation = *pos;
                        }
                    }
                    UnitCommand::MoveToEntity(_) => {}
                    UnitCommand::Completed => {}
                }
            }
        }
    }
}

#[derive(Component)]
pub struct CommandHighlighter;

fn spawn_command_highlighters(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for _ in 0..64 {
        cmd.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(1000000., 9999999., -1.)),
            texture: asset_server.load("icon_plusLarge.png"),
            sprite: Sprite {
                color: Color::srgba(0., 1., 0., 0.1),
                ..default()
            },
            ..Default::default()
        })
        .insert(CommandHighlighter);
    }
}
