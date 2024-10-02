use crate::materials::{Mineable, MineralResources};
use crate::movement::{Avoidance, FaceMovementDirection};
use crate::selection::{CurrentlySelected, Selectable, Team};
use crate::ui::{spawn_build_order_card, BuildQueueParent};
use crate::AppState;
use crate::DontDestroyOnLoad;
use crate::GamePhase;
use crate::MainCamera;
use crate::MapBoundaries;
use bevy::prelude::*;
use bevy::render::view::visibility::RenderLayers;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;
use rand::Rng;
use std::collections::VecDeque;
use std::f32::consts::FRAC_PI_2;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_command_highlighters); //Temp
        app.add_systems(OnEnter(AppState::InGame), (spawn_units, reset_mastermind));
        app.add_systems(OnEnter(AppState::Menu), spawn_main_menu_units);
        app.add_systems(
            Update,
            (
                command_units,
                move_units,
                bullet_behaviour,
                tick_attack_timers,
                handle_aggressive_pigs,
                handle_mildly_aggressive_pigs,
                enemy_mastermind,
                handle_add_to_build_queue,
                build_requested_units,
            )
                .run_if(in_state(GamePhase::Playing)), //TODO: ONLY RUN THESE SYSTEMS IF APPSTATE == INGAME
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
        app.init_resource::<EnemyBrain>();
        app.init_resource::<BuildQueue>();
    }
}

#[derive(Component)]
pub struct MotherUnit;

#[derive(Resource)]
pub struct EnemyBrain {
    pub current_wave: i32,
    pub time_between_wave: Timer,
}
impl Default for EnemyBrain {
    fn default() -> EnemyBrain {
        EnemyBrain {
            current_wave: 0,
            time_between_wave: Timer::from_seconds(20.0, TimerMode::Once),
        }
    }
}

pub enum BuildOrder {
    Miner(Entity),
    Melee(Entity),
    Ranged(Entity),
}

#[derive(Resource)]
pub struct BuildQueue {
    pub queue: VecDeque<BuildOrder>,
    pub build_time: Timer,
    pub max_request: usize,
}
impl Default for BuildQueue {
    fn default() -> BuildQueue {
        BuildQueue {
            queue: VecDeque::new(),
            build_time: Timer::from_seconds(5.0, TimerMode::Once),
            max_request: 5,
        }
    }
}

fn handle_add_to_build_queue(
    mut build_queue: ResMut<BuildQueue>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut minerals: ResMut<MineralResources>,
    mut commands: Commands,
    query_of_card_parent: Query<Entity, With<BuildQueueParent>>,
    asset_server: Res<AssetServer>,
) {
    for card_parent in query_of_card_parent.iter() {
        if build_queue.queue.len() < build_queue.max_request {
            if keyboard_input.just_pressed(KeyCode::KeyQ) && minerals.mineral >= 10.0 {
                minerals.mineral -= 10.0;
                if let Some(card_entity) =
                    spawn_build_order_card(&mut commands, card_parent, &asset_server, 0)
                {
                    build_queue.queue.push_back(BuildOrder::Miner(card_entity));
                }
            }
            if keyboard_input.just_pressed(KeyCode::KeyW) && minerals.mineral >= 30.0 {
                minerals.mineral -= 30.0;
                if let Some(card_entity) =
                    spawn_build_order_card(&mut commands, card_parent, &asset_server, 1)
                {
                    build_queue.queue.push_back(BuildOrder::Melee(card_entity));
                }
            }
            if keyboard_input.just_pressed(KeyCode::KeyE) && minerals.mineral >= 60.0 {
                minerals.mineral -= 60.0;
                if let Some(card_entity) =
                    spawn_build_order_card(&mut commands, card_parent, &asset_server, 2)
                {
                    build_queue.queue.push_back(BuildOrder::Ranged(card_entity));
                }
            }
        }
    }
}

fn build_requested_units(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    mother_unit: Query<&Transform, With<MotherUnit>>,
    mut build_queue: ResMut<BuildQueue>,
) {
    if build_queue.queue.len() > 0 {
        build_queue.build_time.tick(time.delta());
        if build_queue.build_time.finished() {
            build_queue.build_time.reset();

            let mut rng = rand::thread_rng();
            for unit in mother_unit.iter() {
                let spawn_pos = unit.translation + Vec3::new(rng.gen_range(-30.0..30.0), 60.0, 0.0);

                match build_queue.queue.pop_front().unwrap() {
                    BuildOrder::Miner(ent) => {
                        cmd.entity(ent).despawn_recursive();
                        spawn_miner_ally(&mut cmd, spawn_pos, &asset_server);
                    }
                    BuildOrder::Melee(ent) => {
                        cmd.entity(ent).despawn_recursive();
                        spawn_melee_ally(&mut cmd, spawn_pos, &asset_server)
                    }
                    BuildOrder::Ranged(ent) => {
                        cmd.entity(ent).despawn_recursive();
                        spawn_ranged_ally(&mut cmd, spawn_pos, &asset_server);
                    }
                }
            }
        }
    }
}

fn reset_mastermind(mut enemy_brain: ResMut<EnemyBrain>, mut build_queue: ResMut<BuildQueue>) {
    *enemy_brain = EnemyBrain::default();
    *build_queue = BuildQueue::default();
}

fn enemy_mastermind(
    mut enemy_brain: ResMut<EnemyBrain>,
    boundaries: Res<MapBoundaries>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    enemy_brain.time_between_wave.tick(time.delta());
    if enemy_brain.time_between_wave.finished() {
        enemy_brain.time_between_wave.reset();

        enemy_brain.current_wave += 1;
        let number_of_units = (enemy_brain.current_wave as f64).sqrt();
        let column_count = number_of_units.ceil() as i64;

        let mut column_index = 0;
        let mut row_index = 0;
        let mut rng = rand::thread_rng();

        let spawn_side = rng.gen_range(0..4);
        let mut spawn_pos = Vec3::ZERO;
        match spawn_side {
            0 => {
                spawn_pos = Vec3::new(
                    rng.gen_range(boundaries.x_boundaries.x..boundaries.x_boundaries.y),
                    boundaries.y_boundaries.y + 30.0,
                    0.0,
                );
            }
            1 => {
                spawn_pos = Vec3::new(
                    boundaries.x_boundaries.y + 30.0,
                    rng.gen_range(boundaries.y_boundaries.x..boundaries.y_boundaries.y),
                    0.0,
                );
            }
            2 => {
                spawn_pos = Vec3::new(
                    rng.gen_range(boundaries.x_boundaries.x..boundaries.x_boundaries.y),
                    boundaries.y_boundaries.x - 30.0,
                    0.0,
                );
            }
            3 => {
                spawn_pos = Vec3::new(
                    boundaries.x_boundaries.x - 30.0,
                    rng.gen_range(boundaries.y_boundaries.x..boundaries.y_boundaries.y),
                    0.0,
                );
            }
            _ => {}
        }
        for i in 0..enemy_brain.current_wave {
            column_index += 1;
            if column_index >= column_count {
                row_index += 1;
                column_index = 0;
            }
            if i < 7 {
                spawn_melee_enemy(
                    &mut commands,
                    spawn_pos
                        + Vec3::new(80., 0., 0.) * column_index as f32
                        + Vec3::new(0., -80., 0.) * row_index as f32,
                    &asset_server,
                );
            } else {
                spawn_ranged_enemy(
                    &mut commands,
                    spawn_pos
                        + Vec3::new(80., 0., 0.) * column_index as f32
                        + Vec3::new(0., -80., 0.) * row_index as f32,
                    &asset_server,
                );
            }
        }
    }
}

fn spawn_melee_enemy(cmd: &mut Commands, spawn_pos: Vec3, asset_server: &Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    cmd.spawn(SpatialBundle {
        transform: Transform::from_translation(spawn_pos),
        ..Default::default()
    })
    .insert(Collider::cuboid(25.0, 25.0))
    .insert(Sensor)
    .insert(Selectable)
    .insert(UnitCommandList {
        commands: Vec::new(),
    })
    .insert(Health {
        current: 70.,
        max_health: 70.,
    })
    .insert(Velocity(120.))
    .insert(Team(1))
    .insert(AttackComponent {
        attack_range: 100.,
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
                texture: asset_server.load("units/enemy_A.png"),
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

fn spawn_ranged_enemy(cmd: &mut Commands, spawn_pos: Vec3, asset_server: &Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    cmd.spawn(SpatialBundle {
        transform: Transform::from_translation(spawn_pos),
        ..Default::default()
    })
    .insert(Collider::cuboid(25.0, 25.0))
    .insert(Sensor)
    .insert(Selectable)
    .insert(UnitCommandList {
        commands: Vec::new(),
    })
    .insert(Health {
        current: 70.,
        max_health: 70.,
    })
    .insert(Velocity(100.))
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

fn spawn_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));

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
    for i in -2..2 {
        spawn_miner_ally(
            &mut cmd,
            Vec3::new((i as f32 % 10.) * 100., 45., 0.),
            &asset_server,
        );
    }

    //SPAWN RANGERS
    for i in -1..1 {
        spawn_ranged_ally(
            &mut cmd,
            Vec3::new((i as f32 % 10.) * 100., 45. * i as f32 / 10., 0.),
            &asset_server,
        );
    }
}

#[derive(Component)]
pub struct AggressiveLilPig;

#[derive(Component)]
pub struct MildAggression;

//ATTACK ENEMIES WHEN IN 400.0 range
fn handle_mildly_aggressive_pigs(
    mut aggressive_q: Query<(&mut UnitCommandList, Entity), With<MildAggression>>,
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
                    if closest_enemy_unit.1 < 400.0 {
                        command_list
                            .commands
                            .push(UnitCommand::AttackEntity(enemy_entity));
                    }
                }
            }
        }
    }
}
//ATTACK CLOSEST ENEMY NO MATTER HOW FAR AWAY
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
                                    if diff_vec.length() > 110.0 {
                                        tr.translation +=
                                            diff_vec.normalize() * vel.0 * time.delta_seconds();
                                    }

                                    if diff_vec.length() < 130.0
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
                            //RETURN TO MOTHER WHEN ASTEROID IS OFF
                            for mother_unit_e in mother_unit.iter() {
                                *command = UnitCommand::ReturnCargoToUnit(mother_unit_e, None);
                                break;
                            }
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
                        if diff_vec.length() > 40.0 {
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
    q_unit_command_list: Query<(&UnitCommandList, Entity), Without<CommandHighlighter>>,
    mut command_highlighters: Query<&mut Transform, With<CommandHighlighter>>,
    q_tr: Query<&Transform, Without<CommandHighlighter>>,
    mut gizmos: Gizmos,
) {
    for mut tr in command_highlighters.iter_mut() {
        tr.translation = Vec3::new(1000000., 9999999., -1.);
    }

    let mut all_highlighters = command_highlighters.iter_mut();
    for selected in currently_selected.ent.iter() {
        if let Ok((command, unit_e)) = q_unit_command_list.get(*selected) {
            let mut last_pos = None;
            if command.commands.len() == 1 {
                match &command.commands[0] {
                    UnitCommand::ReturnCargoToUnit(_, _) => {
                        if let Ok(unit_tr) = q_tr.get(unit_e) {
                            last_pos = Some(unit_tr.translation);
                        }
                    }
                    UnitCommand::MineEntity(_) => {
                        if let Ok(unit_tr) = q_tr.get(unit_e) {
                            last_pos = Some(unit_tr.translation);
                        }
                    }
                    _ => {}
                }
            } else {
                if let Ok(unit_tr) = q_tr.get(unit_e) {
                    last_pos = Some(unit_tr.translation);
                }
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
                            if let Ok(enemy_tr) = q_tr.get(*enemy_entity) {
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
                    UnitCommand::MineEntity(mineable_entity) => {
                        if let Some(mut highlighter_tr) = all_highlighters.next() {
                            if let Ok(enemy_tr) = q_tr.get(*mineable_entity) {
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
                    UnitCommand::ReturnCargoToUnit(cargo_base, _previous_mineable) => {
                        if let Some(mut highlighter_tr) = all_highlighters.next() {
                            if let Ok(enemy_tr) = q_tr.get(*cargo_base) {
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

fn spawn_ranged_ally(cmd: &mut Commands, spawn_pos: Vec3, asset_server: &Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    cmd.spawn(SpatialBundle {
        transform: Transform::from_translation(spawn_pos),
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
        current: 150.,
        max_health: 150.,
    })
    .insert(Team(0))
    .insert(AttackComponent {
        attack_range: 300.,
        attack_amount: 10.,
        time_between_attacks: attack_timer.clone(),
    })
    .insert(Avoidance {
        last_frame_pos: Vec3::ZERO,
        currently_avoiding: false,
    })
    .insert(MildAggression)
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

fn spawn_miner_ally(cmd: &mut Commands, spawn_pos: Vec3, asset_server: &Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    cmd.spawn(SpatialBundle {
        transform: Transform::from_translation(spawn_pos),
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
        current: 150.,
        max_health: 150.,
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
        max_carry: 10.0,
        time_between_mine: Timer::from_seconds(0.25, TimerMode::Once),
    })
    .with_children(|parent| {
        parent
            .spawn(SpriteBundle {
                texture: asset_server.load("units/station_A.png"),
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

fn spawn_melee_ally(cmd: &mut Commands, spawn_pos: Vec3, asset_server: &Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.75, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));
    cmd.spawn(SpatialBundle {
        transform: Transform::from_translation(spawn_pos),
        ..Default::default()
    })
    .insert(Collider::cuboid(25.0, 25.0))
    .insert(Sensor)
    .insert(Selectable)
    .insert(Velocity(250.))
    .insert(UnitCommandList {
        commands: Vec::new(),
    })
    .insert(Health {
        current: 200.,
        max_health: 200.,
    })
    .insert(Team(0))
    .insert(AttackComponent {
        attack_range: 100.,
        attack_amount: 10.,
        time_between_attacks: attack_timer.clone(),
    })
    .insert(Avoidance {
        last_frame_pos: Vec3::ZERO,
        currently_avoiding: false,
    })
    .insert(MildAggression)
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

fn spawn_main_menu_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    let mut attack_timer = Timer::from_seconds(0.5, TimerMode::Once);
    attack_timer.tick(std::time::Duration::from_secs(1));

    //MOTHERSHIP
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/station_B.png"),
        sprite: Sprite {
            color: Color::srgba(1., 1., 1., 1.),
            custom_size: Some(Vec2::new(128., 128.)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-500.0, -100., 0.)),
        ..Default::default()
    });
    //Miners
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/station_A.png"),
        transform: Transform::from_translation(Vec3::new(-50.0, -80., 0.)),
        ..Default::default()
    });
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/station_A.png"),
        transform: Transform::from_translation(Vec3::new(500.0, 80., 0.)),
        ..Default::default()
    });
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/station_A.png"),
        transform: Transform::from_translation(Vec3::new(-400.0, -150., 0.)),
        ..Default::default()
    });
    //ASTEROIDS
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/meteor_squareDetailedLarge.png"),
        sprite: Sprite {
            color: Color::srgba(1., 1., 1., 1.),
            custom_size: Some(Vec2::new(150., 150.)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(-800.0, -400., 0.)),
        ..Default::default()
    });
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/meteor_squareDetailedLarge.png"),
        sprite: Sprite {
            color: Color::srgba(1., 1., 1., 1.),
            custom_size: Some(Vec2::new(150., 150.)),
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(700.0, 400., 0.)),
        ..Default::default()
    });
}
