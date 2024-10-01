use crate::materials::{Mineable, MineralResources};
use crate::movement::{Avoidance, FaceMovementDirection};
use crate::selection::{CurrentlySelected, Selectable, Team};
use crate::AppState;
use crate::DontDestroyOnLoad;
use crate::MainCamera;
use bevy::prelude::*;
use bevy::render::view::visibility::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;
use std::f32::consts::FRAC_PI_2;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_command_highlighters); //Temp
        app.add_systems(OnEnter(AppState::InGame), (spawn_units, spawn_enemy_units));
        app.add_systems(
            Update,
            (
                command_units,
                move_units,
                bullet_behaviour,
                tick_attack_timers,
                handle_aggressive_pigs,
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

#[derive(Component)]
pub struct MotherUnit;

fn spawn_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    //SPAWN RANGERS
    for i in -3..3 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(
                (i as f32 % 10.) * 100.,
                45. * i as f32 / 10.,
                0.,
            )),
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
        .insert(Avoidance {
            last_frame_pos: Vec3::ZERO,
            currently_avoiding: false,
        })
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
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/meteor_small.png"),
                    sprite: Sprite {
                        color: Color::srgba(0., 1., 0., 1.),
                        custom_size: Some(Vec2::new(100., 100.)),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(RenderLayers::layer(1));
        });
    }
    //SPAWNMOTHERSHIP
    cmd.spawn(SpatialBundle {
        transform: Transform::from_translation(Vec3::new(0., -100., 0.)),
        ..Default::default()
    })
    .insert(Collider::cuboid(50.0, 50.0))
    .insert(Sensor)
    .insert(Selectable)
    .insert(Velocity(50.))
    .insert(UnitCommandList {
        commands: Vec::new(),
    })
    .insert(Health {
        current: 300.,
        max_health: 300.,
    })
    .insert(Team(0))
    .insert(Avoidance {
        last_frame_pos: Vec3::ZERO,
        currently_avoiding: false,
    })
    .insert(MotherUnit)
    .insert(AttackComponent {
        attack_range: 300.,
        attack_amount: 1.,
        time_between_attacks: attack_timer.clone(),
    })
    .with_children(|parent| {
        parent
            .spawn(SpriteBundle {
                texture: asset_server.load("units/station_B.png"),
                sprite: Sprite {
                    color: Color::srgba(1., 1., 1., 1.),
                    custom_size: Some(Vec2::new(128., 128.)),
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
                transform: Transform::from_translation(Vec3::new(0., -60., 0.)),
                sprite: Sprite {
                    color: Color::srgba(0., 1., 0., 1.),
                    ..default()
                },
                ..Default::default()
            })
            .insert(HealthBar);
        parent
            .spawn(SpriteBundle {
                texture: asset_server.load("units/meteor_small.png"),
                sprite: Sprite {
                    color: Color::srgba(0., 1., 0., 1.),
                    custom_size: Some(Vec2::new(140., 140.)),
                    ..default()
                },
                ..Default::default()
            })
            .insert(RenderLayers::layer(1));
    });

    //SPAWN MINERS
    for i in -3..3 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new((i as f32 % 10.) * 100., 45., 0.)),
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
            attack_range: 50.,
            attack_amount: 10.,
            time_between_attacks: attack_timer.clone(),
        })
        .insert(Avoidance {
            last_frame_pos: Vec3::ZERO,
            currently_avoiding: false,
        })
        .insert(MiningComponent {
            current_carry: 0.0,
            max_carry: 5.0,
            time_between_mine: Timer::from_seconds(0.5, TimerMode::Once),
        })
        .with_children(|parent| {
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/enemy_A.png"),
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
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/meteor_small.png"),
                    sprite: Sprite {
                        color: Color::srgba(0., 1., 0., 1.),
                        custom_size: Some(Vec2::new(100., 100.)),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(RenderLayers::layer(1));
        });
    }
}

/*TODO: Spawn enemies in waves
They should attack closest enemies
*/
#[derive(Component)]
pub struct AggressiveLilPig;

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
            current: 50.,
            max_health: 50.,
        })
        .insert(Velocity(150.))
        .insert(Team(1))
        .insert(AttackComponent {
            attack_range: 200.,
            attack_amount: 10.,
            time_between_attacks: attack_timer.clone(),
        })
        .insert(Avoidance {
            last_frame_pos: Vec3::ZERO,
            currently_avoiding: false,
        })
        .insert(AggressiveLilPig)
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
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/meteor_small.png"),
                    sprite: Sprite {
                        color: Color::srgba(1., 0., 0., 1.),
                        custom_size: Some(Vec2::new(100., 100.)),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(RenderLayers::layer(1));
        });
    }
}

fn handle_aggressive_pigs(
    mut aggressive_q: Query<(&mut UnitCommandList, Entity), With<AggressiveLilPig>>,
    all_units: Query<(&Transform, &Team, Entity)>,
) {
    for (mut command_list, e) in aggressive_q.iter_mut() {
        if command_list.commands.len() == 0 {
            let mut aggressive_pig_pos = None;
            let mut pig_team_nr = None;
            if let Ok((pig_tr, pig_team, _e)) = all_units.get(e) {
                aggressive_pig_pos = Some(pig_tr.translation);
                pig_team_nr = Some(pig_team.0);
            }
            if aggressive_pig_pos != None {
                let mut closest_enemy_unit: (Option<Entity>, f32) = (None, f32::MAX);
                for (unit_tr, unit_team, unit_entity) in all_units.iter() {
                    if unit_team.0 != pig_team_nr.unwrap() {
                        let diff_vec = unit_tr.translation - aggressive_pig_pos.unwrap();
                        if diff_vec.length() < closest_enemy_unit.1 {
                            closest_enemy_unit.1 = diff_vec.length();
                            closest_enemy_unit.0 = Some(unit_entity);
                        }
                    }
                }

                if let Some(enemy_entity) = closest_enemy_unit.0 {
                    command_list
                        .commands
                        .push(UnitCommand::AttackEntity(enemy_entity));
                }
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum UnitCommand {
    MoveToPos(Vec3),
    AttackEntity(Entity),
    MineEntity(Entity),
    ReturnCargoToUnit(Entity, Option<Entity>),
    Completed,
}

#[derive(Component)]
pub struct MiningComponent {
    pub current_carry: f32,
    pub max_carry: f32,
    pub time_between_mine: Timer,
}

#[derive(Component)]
pub struct UnitCommandList {
    commands: Vec<UnitCommand>,
}

#[derive(Component)]
pub struct Velocity(pub f32);

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
    q_mining: Query<&MiningComponent>,
    q_mineable: Query<&Mineable>,
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

        let number_of_units = (currently_selected.ent.len() as f64).sqrt();
        let column_count = number_of_units.ceil() as i64;

        let mut column_index = 0;
        let mut row_index = 0;

        for e in currently_selected.ent.iter() {
            if let Ok(mut unit_command_list) = q_unit_command_list.get_mut(*e) {
                let mut moving_to_unit = false;
                let mut has_mining_comp = false;
                if let Ok(_) = q_mining.get(*e) {
                    has_mining_comp = true;
                }
                for clicked_e in clicked_units.iter() {
                    if !keyboard_input.pressed(KeyCode::ShiftLeft) {
                        unit_command_list.commands = Vec::new();
                    }
                    if e != clicked_e {
                        if let Ok(clicked_team) = q_team.get(*clicked_e) {
                            if clicked_team.0 != 0 {
                                unit_command_list
                                    .commands
                                    .push(UnitCommand::AttackEntity(*clicked_e));
                            }
                        } else if let Ok(_mineable) = q_mineable.get(*clicked_e) {
                            if has_mining_comp {
                                unit_command_list
                                    .commands
                                    .push(UnitCommand::MineEntity(*clicked_e));
                            }
                        }
                        //if not: check if material depot: Add unitcommand::return
                        moving_to_unit = true;
                    }
                }

                if !moving_to_unit {
                    if !keyboard_input.pressed(KeyCode::ShiftLeft) {
                        unit_command_list.commands = Vec::new();
                    }
                    unit_command_list.commands.push(UnitCommand::MoveToPos(
                        click_pos.extend(0.)
                            + Vec3::new(80., 0., 0.) * column_index as f32
                            + Vec3::new(0., -80., 0.) * row_index as f32,
                    ));
                    column_index += 1;
                    if column_index >= column_count {
                        row_index += 1;
                        column_index = 0;
                    }
                }
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
    mut mining_component_q: Query<&mut MiningComponent>,
    mut transforms: Query<(&mut Transform, &GlobalTransform)>,
    mut face_direction_q: Query<&mut FaceMovementDirection>,
    mut commands: Commands,
    mut mineables_q: Query<&mut Mineable>,
    mut mineral_resources: ResMut<MineralResources>,
    mother_unit: Query<Entity, With<MotherUnit>>,
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
                UnitCommand::MineEntity(mineable_entity) => {
                    if let Ok(mut mining_comp) = mining_component_q.get_mut(e) {
                        mining_comp.time_between_mine.tick(time.delta());
                        if let Ok(mut mineable) = mineables_q.get_mut(*mineable_entity) {
                            if mining_comp.current_carry < mining_comp.max_carry {
                                if let Ok(
                                    [(mut tr, _global_tr), (mineable_tr, _mineable_global_tr)],
                                ) = transforms.get_many_mut([e, *mineable_entity])
                                {
                                    for child in children {
                                        if let Ok(mut face_dir) = face_direction_q.get_mut(*child) {
                                            face_dir.face_to_pos = mineable_tr.translation;
                                        }
                                    }
                                    let diff_vec = mineable_tr.translation - tr.translation;
                                    if diff_vec.length() > 30.0 {
                                        tr.translation +=
                                            diff_vec.normalize() * vel.0 * time.delta_seconds();
                                    }

                                    if diff_vec.length() < 50.0
                                        && mining_comp.time_between_mine.finished()
                                        && mineable.amount > 0.
                                    {
                                        mining_comp.time_between_mine.reset();
                                        mining_comp.current_carry += 1.0;
                                        mineable.amount -= 1.0;
                                    }
                                }
                            } else {
                                for mother_unit_e in mother_unit.iter() {
                                    *command = UnitCommand::ReturnCargoToUnit(
                                        mother_unit_e,
                                        Some(*mineable_entity),
                                    );
                                    break;
                                }
                            }
                        } else {
                            *command = UnitCommand::Completed;
                        }
                    } else {
                        *command = UnitCommand::Completed;
                    }
                }
                UnitCommand::ReturnCargoToUnit(cargo_base, last_mineable) => {
                    if let Ok([(mut tr, _global_tr), (cargo_base_tr, _cargo_base_global_tr)]) =
                        transforms.get_many_mut([e, *cargo_base])
                    {
                        let diff_vec = cargo_base_tr.translation - tr.translation;
                        for child in children {
                            if let Ok(mut face_dir) = face_direction_q.get_mut(*child) {
                                face_dir.face_to_pos = cargo_base_tr.translation;
                            }
                        }
                        if diff_vec.length() > 30.0 {
                            tr.translation += diff_vec.normalize() * vel.0 * time.delta_seconds();
                        } else {
                            if let Ok(mut mining_comp) = mining_component_q.get_mut(e) {
                                mineral_resources.mineral += mining_comp.current_carry;
                                mining_comp.current_carry = 0.0;
                                if let Some(last_mine) = last_mineable {
                                    *command = UnitCommand::MineEntity(*last_mine);
                                } else {
                                    *command = UnitCommand::Completed;
                                }
                            } else {
                                *command = UnitCommand::Completed;
                            }
                        }
                    } else {
                        *command = UnitCommand::Completed;
                    }
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
                    UnitCommand::MineEntity(mineable_entity) => {}
                    UnitCommand::ReturnCargoToUnit(cargo_base, _previous_mineable) => {}
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
        .insert(CommandHighlighter)
        .insert(DontDestroyOnLoad);
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
