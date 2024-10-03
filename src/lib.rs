#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use bevy::prelude::*;
mod materials;
mod movement;
mod selection;
mod ui;
mod units;
use bevy::asset::AssetMetaCheck;
use bevy::render::camera::ClearColorConfig;
use bevy::render::view::visibility::RenderLayers;
use bevy::window::{CursorGrabMode, PrimaryWindow, WindowMode};
use bevy_rapier2d::prelude::*;
use rand::Rng;
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
    Won,
}

#[derive(Resource)]
pub struct MapBoundaries {
    pub x_boundaries: Vec2,
    pub y_boundaries: Vec2,
}
impl Default for MapBoundaries {
    fn default() -> MapBoundaries {
        MapBoundaries {
            x_boundaries: Vec2::new(-1900.0, 1900.0),
            y_boundaries: Vec2::new(-1900.0, 1900.0),
        }
    }
}

impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, build_world);
        app.add_systems(OnExit(AppState::InGame), despawn_everything);
        app.add_systems(OnExit(AppState::Menu), despawn_everything);
        app.add_systems(Update, detect_lose.run_if(in_state(GamePhase::Playing)));
        app.add_systems(Update, (draw_rect_for_main_cam, keep_camera_in_bounderies));
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(OnEnter(GamePhase::Playing), cursor_grab);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(OnExit(GamePhase::Playing), cursor_ungrab);
        app.add_systems(Update, check_if_won.run_if(in_state(GamePhase::Playing)));
        app.add_systems(
            Update,
            return_to_main_menu.run_if(in_state(AppState::InGame)),
        );
        app.add_systems(Update, spawn_end_point);
        app.init_resource::<MapBoundaries>();
        app.init_resource::<EndGameTimer>();
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

pub fn run() {
    App::new()
        .add_plugins(
            #[cfg(not(target_arch = "wasm32"))]
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        mode: WindowMode::BorderlessFullscreen,
                        fit_canvas_to_parent: true,
                        ..default()
                    }),

                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
            #[cfg(target_arch = "wasm32")]
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        fit_canvas_to_parent: true,
                        ..default()
                    }),

                    ..default()
                })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .init_state::<AppState>()
        .add_sub_state::<GamePhase>()
        .add_plugins(StartupPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        //.add_plugins(RapierDebugRenderPlugin::default())
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
    mut end_game_timer: ResMut<EndGameTimer>,
    mut camera: Query<&mut Transform, With<MainCamera>>,
) {
    for mut tr in camera.iter_mut() {
        tr.translation = Vec3::ZERO;
    }
    for e in &entities {
        commands.entity(e).despawn_recursive();
    }

    *end_game_timer = EndGameTimer::default();
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

fn keep_camera_in_bounderies(
    mut camera_q: Query<(&Camera, &mut Transform), With<MainCamera>>,
    boundaries: Res<MapBoundaries>,
) {
    for (cam, mut cam_tr) in camera_q.iter_mut() {
        let cam_rect = cam.logical_viewport_rect().unwrap();
        let half_width = cam_rect.width() / 2.0;
        let half_height = cam_rect.height() / 2.0;
        //TODO: ADD CAMERA FRUSTRUM TO IT - so we dont boundarie the middle of the camera!
        if cam_tr.translation.x + half_width > boundaries.x_boundaries.y {
            cam_tr.translation.x = boundaries.x_boundaries.y - half_width;
        } else if cam_tr.translation.x - half_width < boundaries.x_boundaries.x {
            cam_tr.translation.x = boundaries.x_boundaries.x + half_width;
        }
        if cam_tr.translation.y + half_height > boundaries.y_boundaries.y {
            cam_tr.translation.y = boundaries.y_boundaries.y - half_height;
        } else if cam_tr.translation.y - half_height < boundaries.y_boundaries.x {
            cam_tr.translation.y = boundaries.y_boundaries.x + half_height;
        }
    }
}

#[derive(Component)]
pub struct EndPoint;

fn check_if_won(
    mother_unit: Query<&Transform, With<MotherUnit>>,
    end_points: Query<&Transform, (With<EndPoint>, Without<MotherUnit>)>,
    mut main_camera: Query<
        &mut Transform,
        (With<MainCamera>, Without<MotherUnit>, Without<EndPoint>),
    >,
    mut game_phase: ResMut<NextState<GamePhase>>,
) {
    for mother_tr in mother_unit.iter() {
        for end_point_tr in end_points.iter() {
            if (mother_tr.translation - end_point_tr.translation).length() < 50.0 {
                for mut cam_tr in main_camera.iter_mut() {
                    cam_tr.translation = mother_tr.translation;
                }
                game_phase.set(GamePhase::Won);
            }
        }
    }
}

#[derive(Resource)]
pub struct EndGameTimer(Timer);

impl Default for EndGameTimer {
    fn default() -> EndGameTimer {
        EndGameTimer(Timer::from_seconds(60.0 * 5.0, TimerMode::Once))
    }
}

fn spawn_end_point(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
    map_boundaries: Res<MapBoundaries>,
    end_points: Query<Entity, With<EndPoint>>,
    time: Res<Time>,
    mut end_game_timer: ResMut<EndGameTimer>,
    mother_unit_q: Query<&GlobalTransform, With<MotherUnit>>,
) {
    let mut end_point_count = 0;
    for _ in end_points.iter() {
        end_point_count += 1;
        break;
    }

    end_game_timer.0.tick(time.delta());
    if end_game_timer.0.finished() {
        if end_point_count == 0 {
            let mut rng = rand::thread_rng();
            for mother_tr in mother_unit_q.iter() {
                let mut spawn_pos = mother_tr.translation();
                let mut repeat_counter = 0;
                while (mother_tr.translation().truncate() - spawn_pos.truncate()).length() < 200.0
                    || repeat_counter > 20
                {
                    repeat_counter += 1;
                    spawn_pos = Vec3::new(
                        rng.gen_range(
                            (map_boundaries.x_boundaries.x + 100.0)
                                ..(map_boundaries.x_boundaries.y - 100.0),
                        ),
                        rng.gen_range(
                            (map_boundaries.y_boundaries.x + 100.0)
                                ..(map_boundaries.y_boundaries.y - 100.0),
                        ),
                        -10.0,
                    );
                }

                let transform = Transform::from_translation(spawn_pos);
                cmd.spawn(SpatialBundle {
                    transform,
                    ..Default::default()
                })
                .insert(EndPoint)
                .with_children(|parent| {
                    parent
                        .spawn(SpriteBundle {
                            texture: asset_server.load("icon_plusLarge.png"),
                            sprite: Sprite {
                                color: Color::srgba(1., 1., 0., 1.),
                                custom_size: Some(Vec2::new(150., 150.)),
                                ..default()
                            },
                            ..Default::default()
                        })
                        .insert(RenderLayers::from_layers(&[0, 1]));
                });
            }
        }
    }
}

fn return_to_main_menu(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        app_state.set(AppState::Menu);
    }
}
