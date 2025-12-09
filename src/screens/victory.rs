//! The victory screen that appears when a level is completed.

use bevy::prelude::*;

use crate::{
    game::{GameLevel, PlayerLevel, Season},
    screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Victory), spawn_victory_screen);
    app.add_systems(Update, handle_continue.run_if(in_state(Screen::Victory)));
}

fn spawn_victory_screen(mut commands: Commands, season: Res<Season>, game_level: Res<GameLevel>) {
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
                Text::new(format!("You completed {} Level {}", *season, game_level.0)),
                TextFont {
                    font_size: 30.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

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

fn handle_continue(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut season: ResMut<Season>,
    mut game_level: ResMut<GameLevel>,
    mut player_level: ResMut<PlayerLevel>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    if keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Enter) {
        // Increment level
        game_level.0 = game_level.0.saturating_add(1);

        // Cycle season every 10 levels
        if game_level.0 > 10 {
            *season = season.next();
            game_level.0 %= 10;
            player_level.0 += 1;
        }

        info!("Continuing to {} Level {}", *season, game_level.0);

        // Transition to Gameplay to load the next level
        next_state.set(Screen::Gameplay);
    }
}

fn percent(percent: i32) -> Val {
    Val::Percent(percent as f32)
}
