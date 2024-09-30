use crate::movement::MoveForward;
use crate::selection::Selectable;
use crate::units::Health;
use crate::AppState;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

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
        app.add_systems(OnEnter(AppState::InGame), spawn_asetroids);
        app.add_systems(PostUpdate, delete_asteroids);
        app.init_resource::<MineralResources>();
    }
}

#[derive(Component)]
pub struct Mineable {
    pub amount: f32,
}

fn spawn_asetroids(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for i in 0..5 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 300. + 100., -5.0)),
            ..Default::default()
        })
        .insert(Collider::cuboid(25.0, 25.0))
        .insert(Sensor)
        .insert(Selectable)
        .insert(Health {
            current: 100.,
            max_health: 100.,
        })
        .insert(MoveForward { speed: 20. })
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
        });
    }
}

fn delete_asteroids(mut cmd: Commands, mineable_query: Query<(&Mineable, Entity)>) {
    for (mineable, e) in mineable_query.iter() {
        if mineable.amount <= 0.0 {
            cmd.entity(e).despawn_recursive();
        }
    }
}
