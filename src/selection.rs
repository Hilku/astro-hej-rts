use crate::{DontDestroyOnLoad, MainCamera};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Selectable;

pub struct SelectionPlugin;

#[derive(Component)]
pub struct SelectionHighlighter;

#[derive(Resource)]
pub struct CurrentlySelected {
    pub ent: Vec<Entity>,
}

impl Default for CurrentlySelected {
    fn default() -> CurrentlySelected {
        CurrentlySelected { ent: Vec::new() }
    }
}

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_highlighters)
            .add_systems(Update, check_selection)
            .add_systems(PostUpdate, highlight_selected)
            .init_resource::<CurrentlySelected>()
            .init_resource::<RectSelection>();
    }
}
#[derive(PartialEq)]
enum RectSelectState {
    NotSelecting,
    Selecting,
}

#[derive(Resource)]
struct RectSelection {
    state: RectSelectState,
    start_point: Vec2,
    current_point: Vec2,
}
impl Default for RectSelection {
    fn default() -> RectSelection {
        RectSelection {
            state: RectSelectState::NotSelecting,
            start_point: Vec2::ZERO,
            current_point: Vec2::ZERO,
        }
    }
}

#[derive(Component)]
pub struct Team(pub i32);

fn check_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut currently_selected: ResMut<CurrentlySelected>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    rapier_context: Res<RapierContext>,
    mut rect_selection: ResMut<RectSelection>,
    mut gizmos: Gizmos,
    team_q: Query<&Team>,
) {
    //Get world position of mouse
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

    if buttons.just_pressed(MouseButton::Left) {
        rect_selection.start_point = click_pos;
        let mut selected_new_unit = false;

        let filter = QueryFilter::default();
        rapier_context.intersections_with_point(click_pos, filter, |entity| {
            if let Ok(team_of_entity) = team_q.get(entity) {
                if team_of_entity.0 == 0 {
                    if keyboard_input.pressed(KeyCode::ControlLeft) {
                        selected_new_unit = true;

                        if !currently_selected.ent.contains(&entity) {
                            currently_selected.ent.push(entity);
                        } else {
                            currently_selected.ent.retain(|e| *e != entity);
                        }
                    } else {
                        currently_selected.ent = Vec::new();
                        currently_selected.ent.push(entity);
                        selected_new_unit = true;
                    }
                    // Return `false` instead if we want to stop searching for other colliders containing this point.
                }
            }
            true
        });

        if !selected_new_unit && !keyboard_input.pressed(KeyCode::ControlLeft) {
            currently_selected.ent = Vec::new();
        }
    } else if buttons.pressed(MouseButton::Left) {
        if (rect_selection.start_point - click_pos).length() > 10.
            || rect_selection.state == RectSelectState::Selecting
        {
            rect_selection.current_point = click_pos;
            if rect_selection.state != RectSelectState::Selecting {
                rect_selection.state = RectSelectState::Selecting;
            }
            let gizmo_rect = Rect::from_corners(rect_selection.start_point, click_pos);
            gizmos.rect(
                gizmo_rect.center().extend(0.),
                Quat::IDENTITY,
                Vec2::new(gizmo_rect.width(), gizmo_rect.height()),
                Color::srgb(0., 1., 0.),
            );
        }
    }
    if buttons.just_released(MouseButton::Left) {
        if rect_selection.state == RectSelectState::Selecting {
            rect_selection.state = RectSelectState::NotSelecting;
            let gizmo_rect = Rect::from_corners(rect_selection.start_point, click_pos);
            //Test rect intersection
            let shape = Collider::cuboid(gizmo_rect.width() / 2., gizmo_rect.height() / 2.);
            let shape_pos = gizmo_rect.center();
            let shape_rot = 0.;
            let filter = QueryFilter::default();
            rapier_context.intersections_with_shape(
                shape_pos,
                shape_rot,
                &shape,
                filter,
                |entity| {
                    if let Ok(team_of_entity) = team_q.get(entity) {
                        if team_of_entity.0 == 0 {
                            if !currently_selected.ent.contains(&entity) {
                                currently_selected.ent.push(entity);
                            }
                        }
                    }
                    true
                },
            );
        }
    }
}

fn spawn_highlighters(mut cmd: Commands, asset_server: Res<AssetServer>) {
    for _ in 0..64 {
        cmd.spawn(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(1000000., 9999999., -1.)),
            texture: asset_server.load("icon_plusLarge.png"),
            sprite: Sprite {
                color: Color::srgb(0., 0.5, 0.),
                ..default()
            },
            ..Default::default()
        })
        .insert(SelectionHighlighter)
        .insert(DontDestroyOnLoad);
    }
}

fn highlight_selected(
    currently_selected: Res<CurrentlySelected>,
    mut highlighters: Query<&mut Transform, With<SelectionHighlighter>>,
    transforms: Query<&Transform, Without<SelectionHighlighter>>,
) {
    for mut tr in highlighters.iter_mut() {
        tr.translation = Vec3::new(1000000., 9999999., -1.);
    }

    let mut all_highlighters = highlighters.iter_mut();
    for selected in currently_selected.ent.iter() {
        if let Ok(selected_tr) = transforms.get(*selected) {
            if let Some(mut highlighter_tr) = all_highlighters.next() {
                highlighter_tr.translation = selected_tr.translation + Vec3::new(0., 0., -1.);
            }
        }
    }
}
