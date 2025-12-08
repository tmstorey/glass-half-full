//! The game's menus and transitions between them.

mod character_select;
mod credits;
mod main;
mod pause;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        credits::plugin,
        character_select::plugin,
        main::plugin,
        settings::plugin,
        pause::plugin,
    ));
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    CharacterSelect,
    Settings,
    Pause,
}
