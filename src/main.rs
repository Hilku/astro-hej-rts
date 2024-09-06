use bevy::prelude::*;
mod movement;
mod selection;
mod units;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct MainCamera;

pub struct StartupPlugin;

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_world);
    }
}

fn build_world(mut cmd: Commands) {
    cmd.spawn(Camera2dBundle::default()).insert(MainCamera);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(StartupPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(selection::SelectionPlugin)
        .add_plugins(units::UnitsPlugin)
        .add_plugins(movement::MovementPlugin)
        .run();
}
