use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    #[actionlike(DualAxis)]
    Run,
    Jump,
    Menu,
    Select,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(InputManagerPlugin::<Action>::default());
    app.add_systems(Startup, setup_input);
}

fn setup_input(mut commands: Commands) {
    let mut input_map = InputMap::default();

    input_map.insert_dual_axis(Action::Run, VirtualDPad::wasd());
    input_map.insert_dual_axis(Action::Run, VirtualDPad::arrow_keys());
    input_map.insert_dual_axis(Action::Run, GamepadStick::LEFT);

    input_map.insert(Action::Jump, KeyCode::Space);
    input_map.insert(Action::Jump, GamepadButton::South);

    input_map.insert(Action::Menu, KeyCode::Escape);
    input_map.insert(Action::Menu, GamepadButton::Start);

    input_map.insert(Action::Select, KeyCode::Enter);
    input_map.insert(Action::Select, GamepadButton::South);

    commands.spawn(input_map);
}
