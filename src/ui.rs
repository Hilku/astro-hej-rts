use crate::materials::MineralResources;
use crate::AppState;
use crate::GamePhase;
use bevy::color::palettes::basic::*;
use bevy::prelude::*;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_ui);
        app.add_systems(OnEnter(AppState::Menu), setup_menu_ui);
        app.add_systems(
            Update,
            (
                button_system.run_if(in_state(AppState::Menu).or_else(in_state(GamePhase::Lost))),
                update_ui_texts,
            ),
        );
        app.add_systems(
            OnEnter(GamePhase::Lost),
            (setup_lose_screen, destroy_all_ui),
        );
    }
}

fn update_ui_texts(
    mut resource_text: Query<&mut Text, With<ResourceText>>,
    mineral_resource: Res<MineralResources>,
) {
    for mut text in resource_text.iter_mut() {
        text.sections[1].value = format!("{}", mineral_resource.mineral);
    }
}

#[derive(Component)]
struct UnitText;

#[derive(Component)]
struct ResourceText;

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                ..default()
            },
            ..default()
        })
        .insert(UIElement)
        .with_children(|parent| {
            parent
                .spawn(
                    TextBundle::from_sections([
                        TextSection::new("Units: ", TextStyle { ..default() }),
                        TextSection::new("0", TextStyle { ..default() }),
                    ])
                    .with_style(Style {
                        bottom: Val::Px(30.),
                        left: Val::Px(30.),
                        ..default()
                    }),
                )
                .insert(UnitText);
            parent
                .spawn(
                    TextBundle::from_sections([
                        TextSection::new("Mineral: ", TextStyle { ..default() }),
                        TextSection::new("0", TextStyle { ..default() }),
                    ])
                    .with_style(Style {
                        bottom: Val::Px(30.),
                        left: Val::Px(50.),
                        ..default()
                    }),
                )
                .insert(ResourceText);
        });
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn setup_menu_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .insert(UIElement)
        .with_children(|parent| {
            parent.spawn(
                // Here we are able to call the `From` method instead of creating a new `TextSection`.
                // This will use the default font (a minimal subset of FiraMono) and apply the default styling.
                TextBundle::from_sections([TextSection::new(
                    "Astro Battler",
                    TextStyle {
                        font_size: 100.0,
                        ..default()
                    },
                )])
                .with_style(Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    ..default()
                })
                .with_text_justify(JustifyText::Center),
            );

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        top: Val::Px(300.0),
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(ButtonInteraction::StartGame)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Start Game",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn setup_lose_screen(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::SpaceEvenly,
                align_items: AlignItems::Center,
                ..default()
            },
            ..default()
        })
        .insert(UIElement)
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_sections([TextSection::new(
                    "You lost!",
                    TextStyle {
                        font_size: 100.0,
                        ..default()
                    },
                )])
                .with_text_justify(JustifyText::Center)
                .with_style(Style {
                    top: Val::Percent(5.),
                    ..default()
                }),
            );

            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        align_content: AlignContent::Center,
                        justify_self: JustifySelf::Center,
                        bottom: Val::Percent(5.0),
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    border_radius: BorderRadius::MAX,
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .insert(ButtonInteraction::BackToMenu)
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back to Menu",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ButtonInteraction,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, mut color, mut border_color, button_interaction) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();
                match button_interaction {
                    ButtonInteraction::StartGame => {
                        app_state.set(AppState::InGame);
                    }
                    ButtonInteraction::BackToMenu => {
                        app_state.set(AppState::Menu);
                    }
                }
                //TODO: handle each button here?? with a marker component
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

#[derive(Component)]
pub struct UIElement;

fn destroy_all_ui(mut commands: Commands, all_ui: Query<Entity, With<UIElement>>) {
    for e in all_ui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Component)]
pub enum ButtonInteraction {
    StartGame,
    BackToMenu,
}
