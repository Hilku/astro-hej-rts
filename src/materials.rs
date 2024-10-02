use std::f32::consts::PI;

use crate::movement::MoveForward;
use crate::selection::Selectable;
use crate::units::Health;
use crate::AppState;
use crate::MapBoundaries;
use bevy::prelude::*;
use bevy::render::view::visibility::RenderLayers;
use bevy_rapier2d::prelude::*;
use rand::Rng;
pub struct MaterialPlugin;

#[derive(Resource)]
pub struct MineralResources {
    pub mineral: f32,
}
impl Default for MineralResources {
    fn default() -> MineralResources {
        MineralResources { mineral: 50.0 }
    }
}

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            (spawn_asetroids, reset_mastermind),
        );
        app.add_systems(PostUpdate, delete_asteroids);
        app.add_systems(Update, asteroid_mastermind);
        app.init_resource::<MineralResources>();
        app.init_resource::<AsteroidBrain>();
    }
}

#[derive(Component)]
pub struct Mineable {
    pub amount: f32,
}

#[derive(Resource)]
pub struct AsteroidBrain {
    pub time_between_wave: Timer,
}
impl Default for AsteroidBrain {
    fn default() -> AsteroidBrain {
        AsteroidBrain {
            time_between_wave: Timer::from_seconds(10.0, TimerMode::Once),
        }
    }
}

fn reset_mastermind(mut asteroid_brain: ResMut<AsteroidBrain>) {
    *asteroid_brain = AsteroidBrain::default();
}
fn asteroid_mastermind(
    mut enemy_brain: ResMut<AsteroidBrain>,
    boundaries: Res<MapBoundaries>,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    enemy_brain.time_between_wave.tick(time.delta());
    if enemy_brain.time_between_wave.finished() {
        enemy_brain.time_between_wave.reset();
        let mut rng = rand::thread_rng();

        let spawn_side = rng.gen_range(0..4);
        let mut spawn_pos = Vec3::ZERO;
        let mut start_rotation = rng.gen_range(PI..(2.0 * PI));
        match spawn_side {
            0 => {
                spawn_pos = Vec3::new(
                    rng.gen_range(boundaries.x_boundaries.x..boundaries.x_boundaries.y),
                    boundaries.y_boundaries.y + 30.0,
                    0.0,
                );
                start_rotation = rng.gen_range(PI..(2.0 * PI));
            }
            1 => {
                spawn_pos = Vec3::new(
                    boundaries.x_boundaries.y + 30.0,
                    rng.gen_range(boundaries.y_boundaries.x..boundaries.y_boundaries.y),
                    0.0,
                );
                start_rotation = rng.gen_range(PI..(2.0 * PI)) - PI / 2.0;
            }
            2 => {
                spawn_pos = Vec3::new(
                    rng.gen_range(boundaries.x_boundaries.x..boundaries.x_boundaries.y),
                    boundaries.y_boundaries.x - 30.0,
                    0.0,
                );
                start_rotation = rng.gen_range(PI..(2.0 * PI)) - 2.0 * (PI / 2.0);
            }
            3 => {
                spawn_pos = Vec3::new(
                    boundaries.x_boundaries.x - 30.0,
                    rng.gen_range(boundaries.y_boundaries.x..boundaries.y_boundaries.y),
                    0.0,
                );
                start_rotation = rng.gen_range(PI..(2.0 * PI)) - 3.0 * (PI / 2.0);
            }
            _ => {}
        }
        spawn_asteroid(&mut commands, &asset_server, spawn_pos, start_rotation);
    }
}
fn spawn_asteroid(
    cmd: &mut Commands,
    asset_server: &Res<AssetServer>,
    spawn_pos: Vec3,
    z_rotation: f32,
) {
    let mut transform = Transform::from_translation(spawn_pos);
    transform.rotation = Quat::from_rotation_z(z_rotation);
    cmd.spawn(SpatialBundle {
        transform,
        ..Default::default()
    })
    .insert(Collider::cuboid(50.0, 50.0))
    .insert(Sensor)
    .insert(Selectable)
    .insert(Health {
        current: 100.,
        max_health: 100.,
    })
    .insert(MoveForward { speed: 40. })
    .insert(Mineable { amount: 50. })
    .with_children(|parent| {
        parent.spawn(SpriteBundle {
            texture: asset_server.load("units/meteor_squareDetailedLarge.png"),
            sprite: Sprite {
                color: Color::srgba(1., 1., 1., 1.),
                custom_size: Some(Vec2::new(150., 150.)),
                ..default()
            },
            ..Default::default()
        });
        parent
            .spawn(SpriteBundle {
                texture: asset_server.load("units/meteor_squareDetailedLarge.png"),
                sprite: Sprite {
                    color: Color::srgba(1., 1., 1., 1.),
                    custom_size: Some(Vec2::new(150., 150.)),
                    ..default()
                },
                ..Default::default()
            })
            .insert(RenderLayers::layer(1));
    });
}

fn spawn_asetroids(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for i in 0..1 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 300. + 100., -5.0)),
            ..Default::default()
        })
        .insert(Collider::cuboid(50.0, 50.0))
        .insert(Sensor)
        .insert(Selectable)
        .insert(Health {
            current: 100.,
            max_health: 100.,
        })
        .insert(MoveForward { speed: 1. })
        .insert(Mineable { amount: 100. })
        .with_children(|parent| {
            parent.spawn(SpriteBundle {
                texture: asset_server.load("units/meteor_squareDetailedLarge.png"),
                sprite: Sprite {
                    color: Color::srgba(1., 1., 1., 1.),
                    custom_size: Some(Vec2::new(150., 150.)),
                    ..default()
                },
                ..Default::default()
            });
            parent
                .spawn(SpriteBundle {
                    texture: asset_server.load("units/meteor_squareDetailedLarge.png"),
                    sprite: Sprite {
                        color: Color::srgba(1., 1., 1., 1.),
                        custom_size: Some(Vec2::new(150., 150.)),
                        ..default()
                    },
                    ..Default::default()
                })
                .insert(RenderLayers::layer(1));
        });
    }
}

fn delete_asteroids(
    mut cmd: Commands,
    mineable_query: Query<(&Mineable, Entity, &Transform)>,
    map_boundaries: Res<MapBoundaries>,
) {
    for (mineable, e, tr) in mineable_query.iter() {
        if mineable.amount <= 0.0 {
            cmd.entity(e).despawn_recursive();
        } else if tr.translation.x < (map_boundaries.x_boundaries.x - 50.0)
            || tr.translation.x > (map_boundaries.x_boundaries.y + 50.0)
            || tr.translation.y < (map_boundaries.y_boundaries.x - 50.0)
            || tr.translation.y > (map_boundaries.y_boundaries.y + 50.0)
        {
            cmd.entity(e).despawn_recursive();
        }
    }
}
