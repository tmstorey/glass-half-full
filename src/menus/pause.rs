//! The pause menu.

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::{
    game::{GameLevel, controls::Action},
    menus::Menu,
    screens::Screen,
    theme::widget,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(Update, go_back.run_if(in_state(Menu::Pause)));
}

fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Pause Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::Pause),
        #[cfg(not(feature = "dev_native"))]
        children![
            widget::header("Game paused"),
            widget::button("Continue", close_menu),
            widget::button("Character Select", open_character_select),
            widget::button("Settings", open_settings_menu),
            widget::button("Quit to title", quit_to_title),
        ],
        #[cfg(feature = "dev_native")]
        children![
            widget::header("Game paused"),
            widget::button("Continue", close_menu),
            widget::button("Character Select", open_character_select),
            widget::button("Settings", open_settings_menu),
            widget::button("Increment season", increment_season),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

fn increment_season(
    _: On<Pointer<Click>>,
    mut game_level: ResMut<GameLevel>,
    mut next_screen: ResMut<NextState<Screen>>,
) {
    game_level.0 = 10;
    next_screen.set(Screen::Victory);
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn open_character_select(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::CharacterSelect);
}

fn close_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None);
}

fn quit_to_title(_: On<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn go_back(action_query: Query<&ActionState<Action>>, mut next_menu: ResMut<NextState<Menu>>) {
    if let Ok(action_state) = action_query.single()
        && action_state.just_pressed(&Action::Menu)
    {
        next_menu.set(Menu::None);
    }
}
