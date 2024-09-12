use bevy::prelude::*;
mod movement;
mod selection;
mod ui;
mod units;
use bevy_rapier2d::prelude::*;
use selection::Team;
use units::MotherUnit;

#[derive(Component)]
pub struct MainCamera;

pub struct StartupPlugin;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum AppState {
    #[default]
    Menu,
    InGame,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug, Default)]
#[source(AppState = AppState::InGame)]
enum GamePhase {
    #[default]
    Playing,
    Lost,
}

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_world);
        app.add_systems(OnExit(AppState::InGame), despawn_everything);
        app.add_systems(OnExit(AppState::Menu), despawn_everything);
    }
}

fn build_world(mut cmd: Commands) {
    cmd.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -1000.0,
            far: 1000.0,
            scale: 1.0,
            ..default()
        },
        ..Default::default()
    })
    .insert(MainCamera)
    .insert(DontDestroyOnLoad);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<AppState>()
        .add_sub_state::<GamePhase>()
        .add_plugins(StartupPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        //  .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(selection::SelectionPlugin)
        .add_plugins(ui::UIPlugin)
        .add_plugins(units::UnitsPlugin)
        .add_plugins(movement::MovementPlugin)
        .run();
}

#[derive(Component)]
struct DontDestroyOnLoad;

fn despawn_everything(
    mut commands: Commands,
    entities: Query<Entity, (Without<Parent>, Without<DontDestroyOnLoad>, Without<Window>)>,
) {
    for e in &entities {
        commands.entity(e).despawn_recursive();
    }
}

fn detect_lose(mother_ship: Query<&Team, With<MotherUnit>>) {}
