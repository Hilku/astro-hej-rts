use bevy::prelude::*;
mod materials;
mod movement;
mod selection;
mod ui;
mod units;
use bevy::render::camera::ClearColorConfig;
use bevy::render::view::visibility::RenderLayers;
use bevy::window::{CursorGrabMode, PrimaryWindow};
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
        app.add_systems(Update, detect_lose.run_if(in_state(GamePhase::Playing)));
        app.add_systems(Update, draw_rect_for_main_cam);
        app.add_systems(OnEnter(GamePhase::Playing), cursor_grab);
        app.add_systems(OnExit(GamePhase::Playing), cursor_ungrab);
    }
}

fn build_world(mut cmd: Commands, mut config_store: ResMut<GizmoConfigStore>) {
    cmd.spawn(Camera2dBundle {
        projection: OrthographicProjection {
            near: -1000.0,
            far: 1000.0,
            scale: 1.0,
            ..default()
        },
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.0, 0.02)),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(MainCamera)
    .insert(DontDestroyOnLoad);
    let (my_config, _) = config_store.config_mut::<MiniMapGizmos>();
    my_config.render_layers = RenderLayers::layer(1);
}

fn draw_rect_for_main_cam(
    mut minimap_gizmos: Gizmos<MiniMapGizmos>,
    camera_q: Query<(&Camera, &Transform), With<MainCamera>>,
) {
    for (cam, cam_tr) in camera_q.iter() {
        let gizmo_rect = cam.logical_viewport_rect().unwrap();
        minimap_gizmos.rect(
            cam_tr.translation,
            Quat::IDENTITY,
            Vec2::new(gizmo_rect.width(), gizmo_rect.height()),
            Color::srgb(0., 1., 0.),
        );
    }
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
        .add_plugins(materials::MaterialPlugin)
        .init_gizmo_group::<MiniMapGizmos>()
        .run();
}
// We can create our own gizmo config group!
#[derive(Default, Reflect, GizmoConfigGroup)]
struct MiniMapGizmos {}
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

//TODO: Add a tracker on the motherunit so we can move the camera there on lose!
fn detect_lose(
    mother_ship: Query<&Team, With<MotherUnit>>,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    let mut has_mother_ship = false;
    for team in mother_ship.iter() {
        if team.0 == 0 {
            has_mother_ship = true;
            break;
        }
    }

    if !has_mother_ship {
        game_phase.set(GamePhase::Lost);
    }
}

fn cursor_grab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();

    // if you want to use the cursor, but not let it leave the window,
    // use `Confined` mode:
    primary_window.cursor.grab_mode = CursorGrabMode::Confined;
}
fn cursor_ungrab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();

    primary_window.cursor.grab_mode = CursorGrabMode::None;
}

/* TODO: add goal! mine 10000 rocks (as many rocks as you can in x time)


*/
