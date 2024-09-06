use crate::MainCamera;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Selectable;

pub struct SelectionPlugin;

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
        app.add_systems(Update, check_selection)
            .init_resource::<CurrentlySelected>();
    }
}

fn check_selection(
    buttons: Res<ButtonInput<MouseButton>>,
    mut currently_selected: ResMut<CurrentlySelected>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    rapier_context: Res<RapierContext>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        //Get world position of click
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
        rapier_context.intersections_with_point(click_pos, filter, |entity| {
            // Callback called on each collider with a shape containing the point.
            println!("The entity {:?} contains the point.", entity);
            // Return `false` instead if we want to stop searching for other colliders containing this point.

            if !currently_selected.ent.contains(&entity) {
                currently_selected.ent.push(entity);
            }
            true
        });
    }
}
