use crate::selection;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use selection::Selectable;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_units); //Temp
    }
}

fn spawn_units(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(SpriteBundle {
        texture: asset_server.load("units/ship_basic.png"),
        transform: Transform::from_translation(Vec3::new(100., 0., 0.)),
        ..Default::default()
    })
    .insert(Collider::cuboid(25.0, 25.0))
    .insert(Sensor)
    .insert(Selectable);
}
