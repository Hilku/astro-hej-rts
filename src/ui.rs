use crate::materials::MineralResources;
use crate::selection::Team;
use crate::units::BuildQueue;
use crate::AppState;
use crate::GamePhase;
use bevy::color::palettes::basic::*;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::visibility::RenderLayers;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_ui);
        app.add_systems(OnEnter(AppState::InGame), setup_minimap);
        app.add_systems(OnEnter(AppState::Menu), setup_menu_ui);
        app.add_systems(
            Update,
            (
                button_system.run_if(in_state(AppState::Menu).or_else(in_state(GamePhase::Lost))),
                update_ui_texts,
                update_unit_ui_texts,
                update_progress_bar,
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

fn update_unit_ui_texts(
    mut resource_text: Query<&mut Text, With<UnitText>>,
    ally_units_q: Query<&Team>,
) {
    let mut count = 0;
    for t in ally_units_q.iter() {
        if t.0 == 0 {
            count += 1;
        }
    }
    for mut text in resource_text.iter_mut() {
        text.sections[1].value = format!("{}", count);
    }
}

#[derive(Component)]
struct UnitText;

#[derive(Component)]
struct ResourceText;

#[derive(Component)]
struct BuildProgressBar;
#[derive(Component)]
pub struct BuildQueueParent;

fn update_progress_bar(
    mut progress_bar: Query<&mut Style, With<BuildProgressBar>>,
    build_queue: Res<BuildQueue>,
) {
    for mut bar_style in progress_bar.iter_mut() {
        bar_style.width = Val::Px(0.0.lerp(256.0, build_queue.build_time.fraction()));
    }
}

pub fn spawn_build_order_card(
    commands: &mut Commands,
    parent: Entity,
    asset_server: &Res<AssetServer>,
    unit_type: i32,
) -> Option<Entity> {
    let mut entity = None;

    commands.entity(parent).with_children(|p| {
        entity = Some(
            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(WHITE.into()),
                ..Default::default()
            })
            .insert(UIElement)
            .id(),
        );
    });
    if let Some(e) = entity {
        commands.entity(e).with_children(|p| {
            p.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(55.0),
                    height: Val::Px(55.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(BLACK.into()),
                ..Default::default()
            })
            .with_children(|pp| {
                let mut asset_path = "";
                if unit_type == 0 {
                    asset_path = "units/station_A.png";
                } else if unit_type == 1 {
                    asset_path = "units/enemy_A.png";
                } else if unit_type == 2 {
                    asset_path = "units/ship_basic.png";
                }

                pp.spawn(ImageBundle {
                    image: UiImage {
                        texture: asset_server.load(asset_path),
                        ..default()
                    },
                    style: Style {
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        ..default()
                    },
                    ..Default::default()
                });
            });
        });
    }

    return entity;
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
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
                        top: Val::Px(20.),
                        left: Val::Px(30.),
                        ..default()
                    }),
                )
                .insert(UnitText);
            parent.spawn(
                TextBundle::from_sections([TextSection::new(" | ", TextStyle { ..default() })])
                    .with_style(Style {
                        top: Val::Px(20.),
                        left: Val::Px(30.),
                        ..default()
                    }),
            );
            parent
                .spawn(
                    TextBundle::from_sections([
                        TextSection::new("Mineral: ", TextStyle { ..default() }),
                        TextSection::new("0", TextStyle { ..default() }),
                    ])
                    .with_style(Style {
                        top: Val::Px(20.),
                        left: Val::Px(30.),
                        ..default()
                    }),
                )
                .insert(ResourceText);
        });
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Component)]
pub struct MinimapCamera;

fn setup_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    asset_server: Res<AssetServer>,
) {
    let size = Extent3d {
        width: 256,
        height: 256,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle = images.add(image);

    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                near: -1000.0,
                far: 1000.0,
                scale: 15.0,
                ..default()
            },
            camera: Camera {
                clear_color: ClearColorConfig::Custom(Color::srgb(0.0, 0.0, 0.1)),
                target: RenderTarget::Image(image_handle.clone()),

                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MinimapCamera)
        .insert(UIElement)
        .insert(RenderLayers::layer(1));

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::ColumnReverse,
                ..default()
            },
            ..default()
        })
        .insert(UIElement)
        .insert(BuildQueueParent)
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage {
                    texture: image_handle,
                    ..Default::default()
                },
                style: Style {
                    width: Val::Px(256.0),
                    height: Val::Px(256.0),
                    ..default()
                },
                ..Default::default()
            });
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Px(256.0),
                        height: Val::Px(20.0),
                        ..default()
                    },
                    background_color: BackgroundColor(YELLOW.into()),
                    ..Default::default()
                })
                .insert(BuildProgressBar);
        });

    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                left: Val::Px(256.0),
                align_items: AlignItems::End,
                ..default()
            },
            ..default()
        })
        .insert(UIElement)
        .with_children(|parent| {
            parent.spawn(ImageBundle {
                image: UiImage {
                    texture: asset_server.load("miner_ally_card.png"),
                    ..default()
                },
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    ..default()
                },
                ..Default::default()
            });
            parent.spawn(ImageBundle {
                image: UiImage {
                    texture: asset_server.load("melee_ally_card.png"),
                    ..default()
                },
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    ..default()
                },
                ..Default::default()
            });
            parent.spawn(ImageBundle {
                image: UiImage {
                    texture: asset_server.load("ranged_ally_card.png"),
                    ..default()
                },
                style: Style {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    ..default()
                },
                ..Default::default()
            });
        });
}

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
