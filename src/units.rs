use crate::movement::{Avoidance, FaceMovementDirection};
use crate::selection::{CurrentlySelected, Selectable, Team};
use crate::MainCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_units, spawn_enemy_units, spawn_command_highlighters),
        ); //Temp
        app.add_systems(
            Update,
            (
                command_units,
                move_units,
                bullet_behaviour,
                tick_attack_timers,
            ),
        );
        app.add_systems(
            PostUpdate,
            (
                display_command_of_selection,
                update_health_bars,
                process_damage_events,
                check_dead_units,
            ),
        );
        app.add_event::<DamageEvent>();
    }
}

fn spawn_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    for i in 0..5 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 0., 0.)),
            ..Default::default()
        })
        .insert(Collider::cuboid(25.0, 25.0))
        .insert(Sensor)
        .insert(Selectable)
        .insert(Velocity(150.))
        .insert(UnitCommandList {
            commands: Vec::new(),
        })
        .insert(Health {
            current: 100.,
            max_health: 100.,
        })
        .insert(Team(0))
        .insert(AttackComponent {
            attack_range: 200.,
            attack_amount: 10.,
            time_between_attacks: attack_timer.clone(),
        })
        .insert(Avoidance)
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/ship_basic.png"),
                    ..Default::default()
                })
                .insert(FaceMovementDirection {
                    face_to_pos: Vec3::ZERO,
                });
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("healthbar.png"),
                    transform: Transform::from_translation(Vec3::new(0., -30., 0.)),
                    sprite: Sprite {
                        color: Color::srgba(0., 1., 0., 1.),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(HealthBar);
        });
    }
}
fn spawn_enemy_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    for i in 0..5 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 300., 0.)),
            ..Default::default()
        })
        .insert(Collider::cuboid(25.0, 25.0))
        .insert(Sensor)
        .insert(Selectable)
        .insert(UnitCommandList {
            commands: Vec::new(),
        })
        .insert(Health {
            current: 100.,
            max_health: 100.,
        })
        .insert(Velocity(150.))
        .insert(Team(1))
        .insert(AttackComponent {
            attack_range: 200.,
            attack_amount: 10.,
            time_between_attacks: attack_timer.clone(),
        })
        .insert(Avoidance)
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/ship_basic.png"),
                    sprite: Sprite {
                        color: Color::srgb(1., 0.5, 0.5),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(FaceMovementDirection {
                    face_to_pos: Vec3::ZERO,
                });
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("healthbar.png"),
                    transform: Transform::from_translation(Vec3::new(0., -30., 0.)),
                    sprite: Sprite {
                        color: Color::srgba(1., 0., 0., 1.),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(HealthBar);
        });
    }
}
#[derive(Component, Clone, Copy)]
pub enum UnitCommand {
    MoveToPos(Vec3),
    AttackEntity(Entity),
    Completed,
}

#[derive(Component)]
pub struct UnitCommandList {
    commands: Vec<UnitCommand>,
}

#[derive(Component)]
struct Velocity(f32);

#[derive(Component)]
pub struct AttackComponent {
    attack_amount: f32,
    attack_range: f32,
    time_between_attacks: Timer,
}

fn command_units(
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    currently_selected: Res<CurrentlySelected>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    rapier_context: Res<RapierContext>,
    mut q_unit_command_list: Query<&mut UnitCommandList>,
    q_team: Query<&Team>,
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
            if let Ok(mut unit_command_list) = q_unit_command_list.get_mut(*e) {
                let mut moving_to_unit = false;
                for clicked_e in clicked_units.iter() {
                    if e != clicked_e {
                        if let Ok(clicked_team) = q_team.get(*clicked_e) {
                            if clicked_team.0 != 0 {
                                if !keyboard_input.pressed(KeyCode::ShiftLeft) {
                                    unit_command_list.commands = Vec::new();
                                }
                                unit_command_list
                                    .commands
                                    .push(UnitCommand::AttackEntity(*clicked_e));
                            }
                        }
                        moving_to_unit = true;
                    }
                }

                if !moving_to_unit {
                    if !keyboard_input.pressed(KeyCode::ShiftLeft) {
                        unit_command_list.commands = Vec::new();
                    }
                    unit_command_list.commands.push(UnitCommand::MoveToPos(
                        click_pos.extend(0.)
                            + Vec3::new(80., 0., 0.) * index as f32
                            + Vec3::new(0., -40., 0.) * index.abs() as f32, //NAIVE formation
                    ))
                }
                index += 1;
            }
        }
    }
}

fn move_units(
    time: Res<Time>,
    mut units: Query<(
        Entity,
        &Velocity,
        &mut UnitCommandList,
        &mut AttackComponent,
        &Children,
    )>,
    mut transforms: Query<(&mut Transform, &GlobalTransform)>,
    mut face_direction_q: Query<&mut FaceMovementDirection>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (e, vel, mut command_list, mut attack_comp, children) in units.iter_mut() {
        if command_list.commands.len() > 0 {
            let command = &mut command_list.commands[0];
            match command {
                UnitCommand::MoveToPos(pos) => {
                    if let Ok((mut tr, _)) = transforms.get_mut(e) {
                        let dif_vec = *pos - tr.translation;
                        if dif_vec.length() > 2. {
                            tr.translation += dif_vec.normalize() * vel.0 * time.delta_seconds();
                            for child in children {
                                if let Ok(mut face_dir) = face_direction_q.get_mut(*child) {
                                    face_dir.face_to_pos = *pos;
                                    break;
                                }
                            }
                        } else {
                            *command = UnitCommand::Completed;
                        }
                    }
                }
                UnitCommand::Completed => {
                    command_list.commands.remove(0);
                }
                UnitCommand::AttackEntity(enemy) => {
                    if let Ok([(mut tr, _global_tr), (enemy_tr, enemy_global_tr)]) =
                        transforms.get_many_mut([e, *enemy])
                    {
                        let unit_translation = tr.translation;
                        let enemy_translation = enemy_tr.translation;
                        let diff_vec = enemy_translation - unit_translation;
                        for child in children {
                            if let Ok(mut face_dir) = face_direction_q.get_mut(*child) {
                                face_dir.face_to_pos = enemy_global_tr.translation();
                                break;
                            }
                        }
                        if diff_vec.length() > attack_comp.attack_range {
                            tr.translation += diff_vec.normalize() * vel.0 * time.delta_seconds();
                        } else {
                            if attack_comp.time_between_attacks.finished() {
                                attack_comp.time_between_attacks.reset();

                                spawn_bullet(
                                    &mut commands,
                                    attack_comp.attack_amount,
                                    tr.translation - Vec3::new(0., 0., 1.),
                                    e,
                                    *enemy,
                                    enemy_tr.translation,
                                    &asset_server,
                                );
                            }
                        }
                    } else {
                        *command = UnitCommand::Completed;
                    }
                }
                _ => {
                    println!("Assigned command i cannot yet do!");
                }
            }
        }
    }
}

fn spawn_bullet(
    cmd: &mut Commands,
    damage: f32,
    spawn_pos: Vec3,
    shooter: Entity,
    target: Entity,
    target_pos: Vec3,
    asset_server: &Res<AssetServer>,
) {
    let mut start_transform = Transform::from_translation(spawn_pos);
    start_transform.scale = Vec3::new(0.1, 0.3, 1.);

    let diff = target_pos - spawn_pos;
    let angle = diff.y.atan2(diff.x) - FRAC_PI_2;
    start_transform.rotation = Quat::from_axis_angle(Vec3::Z, angle);

    cmd.spawn(SpriteBundle {
        texture: asset_server.load("effect_yellow.png"),
        transform: start_transform,
        ..Default::default()
    })
    .insert(FlyingBullet {
        target: target,
        damage: damage,
        speed: 1000.,
        shooter: shooter,
    })
    .insert(FaceMovementDirection {
        face_to_pos: target_pos,
    });
}

#[derive(Component)]
pub struct FlyingBullet {
    target: Entity,
    damage: f32,
    speed: f32,
    shooter: Entity,
}

fn bullet_behaviour(
    time: Res<Time>,
    mut bullets: Query<(&mut Transform, &FlyingBullet, Entity)>,
    targets: Query<&Transform, Without<FlyingBullet>>,
    mut damage_event_writer: EventWriter<DamageEvent>,
    mut cmd: Commands,
) {
    for (mut bullet_tr, bullet, e) in bullets.iter_mut() {
        if let Ok(target_tr) = targets.get(bullet.target) {
            let diff_vec = (target_tr.translation - Vec3::new(0., 0., 1.)) - bullet_tr.translation;
            if diff_vec.length() > 40. {
                bullet_tr.translation +=
                    diff_vec.normalize_or_zero() * time.delta_seconds() * bullet.speed;
            } else {
                cmd.entity(e).despawn_recursive();
                damage_event_writer.send(DamageEvent {
                    target: bullet.target,
                    dmg_amount: bullet.damage,
                    damager: bullet.shooter,
                });
            }
        } else {
            cmd.entity(e).despawn_recursive();
        }
    }
}

fn display_command_of_selection(
    currently_selected: Res<CurrentlySelected>,
    q_unit_command_list: Query<(&UnitCommandList, &Transform), Without<CommandHighlighter>>,
    mut command_highlighters: Query<&mut Transform, With<CommandHighlighter>>,
    mut gizmos: Gizmos,
) {
    for mut tr in command_highlighters.iter_mut() {
        tr.translation = Vec3::new(1000000., 9999999., -1.);
    }

    let mut all_highlighters = command_highlighters.iter_mut();
    for selected in currently_selected.ent.iter() {
        if let Ok((command, unit_tr)) = q_unit_command_list.get(*selected) {
            let mut last_pos = None;
            if command.commands.len() > 1 {
                last_pos = Some(unit_tr.translation);
            }
            for c in &command.commands {
                match c {
                    UnitCommand::MoveToPos(pos) => {
                        if let Some(mut highlighter_tr) = all_highlighters.next() {
                            highlighter_tr.translation = *pos;
                            if let Some(last_p) = last_pos {
                                gizmos.linestrip([last_p, *pos], Color::srgba(0., 1., 0., 0.1));
                            }
                            last_pos = Some(*pos);
                        }
                    }
                    UnitCommand::AttackEntity(enemy_entity) => {
                        if let Some(mut highlighter_tr) = all_highlighters.next() {
                            if let Ok((_, enemy_tr)) = q_unit_command_list.get(*enemy_entity) {
                                highlighter_tr.translation =
                                    enemy_tr.translation - Vec3::new(0., 0., 1.);
                                if let Some(last_p) = last_pos {
                                    gizmos.linestrip(
                                        [last_p, enemy_tr.translation],
                                        Color::srgba(0., 1., 0., 0.1),
                                    );
                                }
                                last_pos = Some(enemy_tr.translation);
                            }
                        }
                    }
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

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max_health: f32,
}
#[derive(Event)]
pub struct DamageEvent {
    target: Entity,
    dmg_amount: f32,
    damager: Entity,
}

fn update_health_bars(
    mut health_q: Query<(&mut Health, &Children)>,
    mut healthbar_q: Query<&mut Transform, With<HealthBar>>,
) {
    for (health, children) in health_q.iter_mut() {
        for c in children.iter() {
            if let Ok(mut bar) = healthbar_q.get_mut(*c) {
                bar.scale = Vec3::new(health.current / health.max_health, 1., 1.);
            }
        }
    }
}

fn process_damage_events(
    mut ev_damage: EventReader<DamageEvent>,
    mut health_q: Query<&mut Health>,
    mut unit_commands: Query<&mut UnitCommandList>,
) {
    for dmg_event in ev_damage.read() {
        if let Ok(mut hp) = health_q.get_mut(dmg_event.target) {
            hp.current -= dmg_event.dmg_amount;
            hp.current = hp.current.clamp(0., hp.max_health);
            if let Ok(mut unit_command) = unit_commands.get_mut(dmg_event.target) {
                if unit_command.commands.len() == 0 {
                    unit_command
                        .commands
                        .push(UnitCommand::AttackEntity(dmg_event.damager));
                }
            }
        }
    }
}

fn check_dead_units(mut cmd: Commands, health: Query<(&Health, Entity)>) {
    for (hp, e) in health.iter() {
        if hp.current <= 0. {
            cmd.entity(e).despawn_recursive();
        }
    }
}

fn tick_attack_timers(time: Res<Time>, mut attack_comps: Query<&mut AttackComponent>) {
    for mut attack_comp in attack_comps.iter_mut() {
        attack_comp.time_between_attacks.tick(time.delta());
    }
}
