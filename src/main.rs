mod buttons;

use std::time::Duration;

use bevy::log;
use bevy::log::{Level, LogSettings};
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use board_plugin::components::Uncover;
use board_plugin::events::{BoardCompletedEvent, BombExplosionEvent};

use crate::buttons::{ButtonAction, ButtonColors};
#[cfg(feature = "debug")]
use bevy_inspector_egui::RegisterInspectable;
use board_plugin::{Board, BoardAssets, BoardOptions, BoardPlugin, BoardPosition, SpriteMaterial};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    InGame,
    Out,
}

#[derive(Default)]
pub struct Cheating {
    pub count: u32,
}
pub struct StartTime {
    pub epoch: f64,
}
impl StartTime {
    pub fn new(epoch: f64) -> Self {
        StartTime { epoch }
    }
}

#[derive(Component)]
pub struct CheatUI;

#[derive(Component)]
pub struct TimeUI;

#[derive(Component)]
pub struct BombCountUI;

fn main() {
    let mut app = App::new();
    // Window setup
    app.insert_resource(WindowDescriptor {
        title: "Mine Sweeper!".to_string(),
        width: 700.,
        height: 750.,
        ..Default::default()
    })
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::ReactiveLowPower {
            max_wait: Duration::new(1, 0),
        },
        unfocused_mode: UpdateMode::ReactiveLowPower {
            max_wait: Duration::new(1, 0),
        },
        ..Default::default()
    })
    // Log setup
    .insert_resource(LogSettings {
        level: Level::INFO,
        ..Default::default()
    })
    // Bevy default plugins
    .add_plugins(DefaultPlugins);
    // Debug hierarchy inspector
    #[cfg(feature = "debug")]
    {
        app.add_plugin(bevy_inspector_egui::WorldInspectorPlugin::new());
        app.register_inspectable::<ButtonAction>();
    }
    // Board plugin
    app.add_plugin(BoardPlugin {
        running_state: AppState::InGame,
    })
    .add_state(AppState::Out)
    .add_system_set(SystemSet::on_update(AppState::Out).with_system(setup_board))
    // Startup system (cameras)
    .add_startup_system(setup_camera)
    // UI
    .add_startup_system(setup_ui)
    // State handling
    .add_system(input_handler)
    .add_system(update_ui)
    .add_system(check_end_of_game)
    // Run the app
    .run();
}

fn setup_board(
    mut commands: Commands,
    mut state: ResMut<State<AppState>>,
    asset_server: Res<AssetServer>,
    mut run_state: Local<u8>,
) {
    match *run_state {
        0 => {
            // Board plugin options
            commands.insert_resource(BoardOptions {
                map_size: (30, 16),
                bomb_count: 99,
                tile_padding: 1.,
                safe_start: true,
                position: BoardPosition::Centered {
                    offset: Vec3::new(0., 25., 0.),
                },
                ..Default::default()
            });
            // Board assets
            commands.insert_resource(BoardAssets {
                label: "Default".to_string(),
                board_material: SpriteMaterial {
                    color: Color::WHITE,
                    ..Default::default()
                },
                tile_material: SpriteMaterial {
                    color: Color::DARK_GRAY,
                    ..Default::default()
                },
                covered_tile_material: SpriteMaterial {
                    color: Color::GRAY,
                    ..Default::default()
                },
                bomb_counter_font: asset_server.load("fonts/pixeled.ttf"),
                bomb_counter_colors: BoardAssets::default_colors(),
                flag_material: SpriteMaterial {
                    texture: asset_server.load("sprites/flag.png"),
                    color: Color::WHITE,
                },
                bomb_material: SpriteMaterial {
                    texture: asset_server.load("sprites/bomb.png"),
                    color: Color::WHITE,
                },
            });
            *run_state = 1;
            bevy::log::info!("Loaded assets");
        },
        1 => {
            // Launch game
            bevy::log::info!("Switch to ingame");
            if *state.current() != AppState::InGame {
                state.set(AppState::InGame).unwrap();
                *run_state = 2;
            }
        },
        _ => {}
    }
}

fn setup_camera(mut commands: Commands) {
    // 2D orthographic camera
    commands.spawn_bundle(Camera2dBundle::default());
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
fn input_handler(
    mut commands: Commands,
    button_colors: Res<ButtonColors>,
    mut interaction_query: Query<
        (&Interaction, &ButtonAction, &mut UiColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut state: ResMut<State<AppState>>,
    mut board: Option<ResMut<Board>>,
    mut cheating: ResMut<Cheating>,
    mut start_time: ResMut<StartTime>,
    time: Res<Time>,
) {
    for (interaction, action, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Clicked => {
                *color = button_colors.pressed.into();
                match action {
                    ButtonAction::Clear => {
                        log::debug!("clearing detected");
                        if state.current() == &AppState::InGame {
                            log::info!("clearing game");
                            state.set(AppState::Out).unwrap();
                        }
                    }
                    ButtonAction::Generate => {
                        log::debug!("loading detected");
                        if state.current() == &AppState::Out {
                            log::info!("loading game");
                            if state.current() == &AppState::Out {
                                cheating.count = 0;
                                start_time.epoch = time.seconds_since_startup();
                            }
                            state.set(AppState::InGame).unwrap();
                        }
                    }
                    &ButtonAction::Cheat => {
                        if let Some(ref mut board) = board {
                            if let Some(coord) = board.find_safe_covered_coord() {
                                for entity in board.tile_to_uncover(&coord) {
                                    commands.entity(entity).insert(Uncover);
                                    cheating.count += 1;
                                }
                            }
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *color = button_colors.hovered.into();
            }
            Interaction::None => {
                *color = button_colors.normal.into();
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn update_ui(
    mut query: ParamSet<(
        Query<&mut Text, With<CheatUI>>,
        Query<&mut Text, With<TimeUI>>,
        Query<&mut Text, With<BombCountUI>>,
    )>,
    cheating: Res<Cheating>,
    //mut time_text_query: Query<&mut Text, (With<TimeUI>, Without<CheatUI>)>,
    start_time: Res<StartTime>,
    time: Res<Time>,
    state: Res<State<AppState>>,
    board: Option<Res<Board>>,
) {
    if state.current() == &AppState::InGame {
        if let Ok(mut cheat_text) = query.p0().get_single_mut() {
            cheat_text.sections[0].value = format!("Cheats: {}", cheating.count);
        }
        if let Ok(mut time_text) = query.p1().get_single_mut() {
            let time_passed = (time.seconds_since_startup() - start_time.epoch) as u32;
            let seconds = time_passed % 60;
            let minutes = time_passed / 60;
            time_text.sections[0].value = format!("Time: {minutes}:{seconds:02}");
        }
        if let (Ok(mut bomb_count_text), Some(board)) = (query.p2().get_single_mut(), board) {
            let bomb_count = board.tile_map.bomb_count();
            let marked_fields = board.marked_tiles.len();
            bomb_count_text.sections[0].value = format!("{marked_fields}/{bomb_count}");
        }
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>, time: Res<Time>) {
    let button_materials = ButtonColors {
        normal: Color::GRAY,
        hovered: Color::DARK_GRAY,
        pressed: Color::BLACK,
    };
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Px(50.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    color: Color::WHITE.into(),
                    ..Default::default()
                })
                .insert(Name::new("UI"))
                .with_children(|parent| {
                    let font = asset_server.load("fonts/pixeled.ttf");
                    setup_single_menu(
                        parent,
                        "CLEAR",
                        button_materials.normal.into(),
                        font.clone(),
                        ButtonAction::Clear,
                    );
                    setup_single_menu(
                        parent,
                        "CHEAT",
                        button_materials.normal.into(),
                        font.clone(),
                        ButtonAction::Cheat,
                    );
                    setup_single_menu(
                        parent,
                        "GENERATE",
                        button_materials.normal.into(),
                        font,
                        ButtonAction::Generate,
                    );
                });
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.), Val::Px(50.)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::SpaceBetween,
                        ..Default::default()
                    },
                    color: Color::WHITE.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    let font = asset_server.load("fonts/pixeled.ttf");
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::from_section(
                                "Cheats: 0".to_string(),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 60.0,
                                    color: Color::BLACK,
                                }).with_alignment(
                                    TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    }
                                ),
                            ..Default::default()
                        })
                        .insert(CheatUI);
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::from_section(
                                "0/0".to_string(),
                                TextStyle {
                                    font: font.clone(),
                                    font_size: 60.0,
                                    color: Color::BLACK,
                                }).with_alignment(
                                TextAlignment {
                                    vertical: VerticalAlign::Center,
                                    horizontal: HorizontalAlign::Center,
                                }
                            ),
                            ..Default::default()
                        })
                        .insert(BombCountUI);
                    parent
                        .spawn_bundle(TextBundle {
                            text: Text::from_section(
                                "Time: 0:00".to_string(),
                                TextStyle {
                                    font,
                                    font_size: 60.0,
                                    color: Color::BLACK,
                                }).with_alignment(
                                    TextAlignment {
                                        vertical: VerticalAlign::Center,
                                        horizontal: HorizontalAlign::Center,
                                    }
                            ),
                            ..Default::default()
                        })
                        .insert(TimeUI);
                });
        });
    commands.insert_resource(button_materials);
    commands.insert_resource(Cheating::default());
    commands.insert_resource(StartTime::new(time.seconds_since_startup()));
}

fn setup_single_menu(
    parent: &mut ChildBuilder,
    text: &str,
    color: UiColor,
    font: Handle<Font>,
    action: ButtonAction,
) {
    parent
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Percent(95.), Val::Auto),
                margin: UiRect::all(Val::Px(10.)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            color,
            ..Default::default()
        })
        .insert(action)
        .insert(Name::new(text.to_string()))
        .with_children(|builder| {
            builder.spawn_bundle(TextBundle {
                text: Text {
                    sections: vec![TextSection {
                        value: text.to_string(),
                        style: TextStyle {
                            font,
                            font_size: 30.,
                            color: Color::WHITE,
                        },
                    }],
                    alignment: TextAlignment {
                        vertical: VerticalAlign::Center,
                        horizontal: HorizontalAlign::Center,
                    },
                },
                ..Default::default()
            });
        });
}

fn check_end_of_game(
    mut commands: Commands,
    mut win_events: EventReader<BoardCompletedEvent>,
    mut bomb_explode_events: EventReader<BombExplosionEvent>,
    mut state: ResMut<State<AppState>>,
    board: Option<Res<Board>>,
) {
    if win_events.iter().next().is_some() || bomb_explode_events.iter().next().is_some() {
        state.push(AppState::Out).unwrap();
        if let Some(board) = board {
            board
                .get_all_bomb_coordinates()
                .iter()
                .filter_map(|coordinate| board.covered_tiles.get(coordinate))
                .for_each(|entity| {
                    bevy::log::info!("Uncover bomb");
                    commands.entity(*entity).insert(Uncover);
                });
        }
    }
    //bevy::log::info!("Frame update");
}
