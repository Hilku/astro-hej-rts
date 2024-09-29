use crate::movement::MoveForward;
use crate::selection::{Selectable, Team};
use crate::units::Health;
use crate::AppState;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), spawn_asetroids);
    }
}

#[derive(Component)]
pub struct Mineable {
    pub amount: f32,
}

fn spawn_asetroids(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for i in 0..5 {
        cmd.spawn(SpatialBundle {
            transform: Transform::from_translation(Vec3::new(i as f32 * 100., 300. + 100., 0.)),
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

/*TODO: Add similar system to space miner: show visuals of mineable nodes but it should be fully managed! -> mined node should spawn a visual node ->
that gets added to the list of the inventory component of the unit -> if returning mined amount to base -> it should remove materials from you (so we should basically just parent it under the unit's sprite when adding it to the unit)
when returning-> check all children of the sprite -> if it has "material" component, add it to the material resource*/
