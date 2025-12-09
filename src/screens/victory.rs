//! The victory screen that appears when a level is completed.

use bevy::prelude::*;

use crate::{
    game::{
        CompletedYear, GameLevel, PlayerLevel, Season,
        character::{COLUMNS, ROWS},
    },
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Victory), spawn_victory_screen);
    app.add_systems(Update, handle_continue.run_if(in_state(Screen::Victory)));
}

fn spawn_victory_screen(
    mut commands: Commands,
    season: Res<Season>,
    game_level: Res<GameLevel>,
    completed_year: Res<CompletedYear>,
    asset_server: Res<AssetServer>,
) {
    let complete_message = if game_level.0 == 10 {
        if *season == Season::Spring {
            "You completed a whole year!".to_string()
        } else {
            format!("You completed all of {}!", *season)
        }
    } else {
        format!("You completed {} Level {}", *season, game_level.0)
    };

    commands
        .spawn((
            Name::new("Victory Screen"),
            Node {
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
            DespawnOnExit(Screen::Victory),
        ))
        .with_children(|parent| {
            // Victory title
            parent.spawn((
                Name::new("Victory Title"),
                Text::new("Level Complete!"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Level info
            parent.spawn((
                Name::new("Level Info"),
                Text::new(complete_message),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if game_level.0 == 10 {
                parent.spawn((
                    Name::new("Unlocked Item Title"),
                    Text::new("Unlocked clothing items:"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
                parent.spawn(unlocked_items(
                    *season,
                    completed_year.0,
                    asset_server.clone(),
                ));
            }

            // Continue instruction
            parent.spawn((
                Name::new("Continue Instruction"),
                Text::new("Press SPACE or ENTER to continue"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn unlocked_items(season: Season, completed_year: bool, asset_server: AssetServer) -> impl Bundle {
    let items = if !completed_year {
        match season {
            Season::Summer => {
                vec![
                    "clothes/fancy-dress",
                    "headwear/farming-hat",
                    "headwear/mining-helmet",
                ]
            }
            Season::Autumn => {
                vec![
                    "footwear/socks/blue",
                    "clothes/queen-dress",
                    "headwear/witch-hat",
                ]
            }
            Season::Winter => {
                vec![
                    "footwear/thighhighs/1",
                    "clothes/skirt",
                    "headwear/santa-hat",
                ]
            }
            Season::Spring => {
                vec![
                    "underclothes/bikini/blue",
                    "clothes/short-skirt",
                    "headwear/bunnyears/1",
                ]
            }
        }
    } else if season == Season::Summer {
        vec!["underclothes/underwear/blue"]
    } else {
        vec![]
    };
    let layout = asset_server.add(TextureAtlasLayout::from_grid(
        UVec2::new(80, 64),
        COLUMNS,
        ROWS,
        None,
        None,
    ));
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: px(10),
            row_gap: px(10),
            flex_wrap: FlexWrap::Wrap,
            max_width: px(800),
            justify_content: JustifyContent::Center,
            margin: UiRect::bottom(px(20)),
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for item in items {
                let path = format!("images/character/{}.epng", item);
                let texture = asset_server.load(path);
                parent.spawn((
                    ImageNode {
                        image: texture,
                        texture_atlas: Some(TextureAtlas {
                            layout: layout.clone(),
                            index: 0, // Idle pose, first frame
                        }),
                        ..default()
                    },
                    UiTransform::from_scale(Vec2::splat(1.5)),
                ));
            }
        })),
    )
}

fn handle_continue(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut season: ResMut<Season>,
    mut game_level: ResMut<GameLevel>,
    mut player_level: ResMut<PlayerLevel>,
    mut completed_year: ResMut<CompletedYear>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        // Increment level
        game_level.0 = game_level.0.saturating_add(1);

        if game_level.0 > 10 {
            if !completed_year.0 {
                player_level.0 += 1;
                if *season == Season::Spring {
                    completed_year.0 = true;
                }
            } else if completed_year.0 && *season == Season::Summer {
                player_level.0 += 1;
            }
            *season = season.next();
            game_level.0 %= 10;
        }

        info!("Continuing to {} Level {}", *season, game_level.0);

        // Transition to Gameplay to load the next level
        next_state.set(Screen::Gameplay);
    }
}

fn percent(percent: i32) -> Val {
    Val::Percent(percent as f32)
}
