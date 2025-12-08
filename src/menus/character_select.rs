use crate::{
    game::{
        PlayerLevel,
        character::{COLUMNS, CharacterLayer, CharacterLayers, LayerType, LayerVariant, ROWS},
        controls::Action,
    },
    menus::Menu,
    screens::Screen,
    theme::{palette::*, prelude::*},
};
use bevy::{
    ecs::{relationship::RelatedSpawner, spawn::SpawnWith},
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::{HoverMap, Hovered},
    prelude::*,
    ui_widgets::{ControlOrientation, CoreScrollbarDragState, CoreScrollbarThumb, Scrollbar},
};
use leafwing_input_manager::prelude::*;
use std::sync::LazyLock;

pub static BODY_COLOURS: LazyLock<Vec<Color>> = LazyLock::new(|| {
    vec![
        Srgba::hex("#f3d0c4").unwrap().into(),
        Srgba::hex("#e9b5a3").unwrap().into(),
        Srgba::hex("#b97b67").unwrap().into(),
        Srgba::hex("#9a5c49").unwrap().into(),
        Srgba::hex("#5b3138").unwrap().into(),
    ]
});

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CharacterSelectState>();

    app.add_systems(OnEnter(Menu::CharacterSelect), spawn_character_select_menu);
    app.add_systems(
        Update,
        (
            go_back,
            handle_layer_type_change,
            handle_item_selection,
            update_character_preview,
            send_scroll_events,
            update_scrollbar_thumb,
        )
            .run_if(in_state(Menu::CharacterSelect)),
    );

    app.add_observer(on_scroll_handler);
}

/// Resource tracking the current state of character customization
#[derive(Resource, Debug)]
struct CharacterSelectState {
    /// Currently selected layer type to customize
    current_layer_type: LayerType,
    /// Current character configuration (all layers)
    current_layers: CharacterLayers,
}

impl Default for CharacterSelectState {
    fn default() -> Self {
        Self {
            current_layer_type: LayerType::Body,
            current_layers: CharacterLayers::default(),
        }
    }
}

/// Marker component for the character preview container
#[derive(Component)]
struct CharacterPreview;

/// Marker component for the items grid
#[derive(Component)]
struct ItemsGrid;

/// Component tagging buttons that change the layer type
#[derive(Component)]
struct LayerTypeButton(LayerType);

/// Component tagging buttons that select an item
#[derive(Component)]
struct ItemButton {
    layer: Option<CharacterLayer>,
    layer_type: LayerType,
}

fn spawn_character_select_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        widget::ui_root("Character Select Menu"),
        GlobalZIndex(2),
        DespawnOnExit(Menu::CharacterSelect),
        children![
            // Upper section: Character preview
            character_preview_section(&asset_server),
            // Middle section: Layer type selector
            layer_type_selector(),
            // Lower section: Items grid
            items_grid_section(),
            // Bottom buttons
            (
                Name::new("Bottom Buttons"),
                Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    column_gap: px(20),
                    ..default()
                },
                children![
                    widget::button("Back", go_back_on_click),
                    widget::button("Confirm", confirm_selection),
                ],
            ),
        ],
    ));
}

fn character_preview_section(_asset_server: &AssetServer) -> impl Bundle {
    (
        Name::new("Character Preview Section"),
        CharacterPreview,
        Node {
            min_width: px(250),
            min_height: px(250),
            padding: UiRect::bottom(px(50)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Srgba::hex("#222222").unwrap().into()),
        BorderRadius::all(px(10)),
        // Child sprites will be added dynamically
        Children::default(),
    )
}

fn layer_type_selector() -> impl Bundle {
    let layer_types = vec![
        LayerType::Body,
        LayerType::Hair,
        LayerType::Underclothes,
        LayerType::Footwear,
        LayerType::Clothes,
        LayerType::Gloves,
        LayerType::Cape,
        LayerType::Headwear,
    ];

    (
        Name::new("Layer Type Selector"),
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: px(10),
            row_gap: px(10),
            flex_wrap: FlexWrap::Wrap,
            max_width: px(800),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            for layer_type in layer_types {
                parent.spawn(layer_type_button(layer_type));
            }
        })),
    )
}

fn layer_type_button(layer_type: LayerType) -> impl Bundle {
    (
        Name::new(format!("{:?} Button", layer_type)),
        LayerTypeButton(layer_type),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Name::new("Button Inner"),
                Button,
                BackgroundColor(*BUTTON_BACKGROUND),
                InteractionPalette {
                    none: *BUTTON_BACKGROUND,
                    hovered: *BUTTON_HOVERED_BACKGROUND,
                    pressed: *BUTTON_PRESSED_BACKGROUND,
                },
                Node {
                    width: px(180),
                    height: px(50),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(px(25)),
                children![(
                    Name::new("Button Text"),
                    Text(format!("{:?}", layer_type)),
                    TextFont::from_font_size(20.0),
                    TextColor(*BUTTON_TEXT),
                    Pickable::IGNORE,
                )],
            ));
        })),
    )
}

fn items_grid_section() -> impl Bundle {
    (
        Name::new("Items Grid Frame"),
        Node {
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
            grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.)],
            max_width: px(820),
            max_height: px(400),
            column_gap: px(2),
            ..default()
        },
        Children::spawn(SpawnWith(|parent: &mut RelatedSpawner<ChildOf>| {
            // The actual scrolling items grid
            let scroll_area_id = parent
                .spawn((
                    Name::new("Items Grid Section"),
                    ItemsGrid,
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::px(6, 100.0),
                        row_gap: px(10),
                        column_gap: px(10),
                        padding: UiRect::all(px(20)),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Srgba::hex("#1a1a1a").unwrap().into()),
                    BorderRadius::all(px(10)),
                    ScrollPosition::default(),
                    Children::default(),
                ))
                .id();

            // Vertical scrollbar
            parent.spawn((
                Node {
                    min_width: px(12),
                    grid_row: GridPlacement::start(1),
                    grid_column: GridPlacement::start(2),
                    ..default()
                },
                Scrollbar {
                    orientation: ControlOrientation::Vertical,
                    target: scroll_area_id,
                    min_thumb_length: 20.0,
                },
                Children::spawn(Spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    Hovered::default(),
                    BackgroundColor(Srgba::hex("#888888").unwrap().into()),
                    BorderRadius::all(px(6)),
                    CoreScrollbarThumb,
                ))),
            ));
        })),
    )
}

fn item_button(
    layer: Option<CharacterLayer>,
    layer_type: LayerType,
    asset_server: &AssetServer,
) -> impl Bundle {
    let asset_server = asset_server.clone();
    let is_body = layer_type == LayerType::Body;
    let (path, variant) = if let Some(layer) = layer.clone() {
        let path = layer
            .texture_path()
            .unwrap_or_else(|_| "images/character/empty.epng".to_string());
        (path, layer.variant)
    } else {
        let path = "images/character/empty.epng".to_string();
        (path, None)
    };
    let texture = asset_server.load(path);
    let layout = TextureAtlasLayout::from_grid(UVec2::new(80, 64), COLUMNS, ROWS, None, None);

    (
        Name::new("Item"),
        ItemButton {
            layer: layer.clone(),
            layer_type,
        },
        Node::default(),
        Pickable::IGNORE,
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Name::new("Button Inner"),
                Button,
                BackgroundColor(*TRANSPARENT),
                InteractionPalette {
                    none: *TRANSPARENT,
                    hovered: *BUTTON_HOVERED_BACKGROUND,
                    pressed: *BUTTON_PRESSED_BACKGROUND,
                },
                Node {
                    width: px(90),
                    height: px(90),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BorderRadius::all(px(2)),
                Pickable::IGNORE,
                // Sprite or color square child
                Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                    if is_body {
                        // For Body type, use a colored square
                        if let Some(LayerVariant::Variant(index)) = variant {
                            let color = BODY_COLOURS
                                .get((index - 1) as usize)
                                .copied()
                                .unwrap_or(Color::WHITE);
                            parent.spawn((
                                Node {
                                    width: px(80),
                                    height: px(80),
                                    ..default()
                                },
                                BackgroundColor(color),
                                Pickable::IGNORE,
                            ));
                        }
                    } else {
                        // For other types, show the first sprite from the spritesheet
                        parent.spawn((
                            Name::new("Item"),
                            Node {
                                width: px(80),
                                height: px(80),
                                position_type: PositionType::Absolute,
                                ..default()
                            },
                            ImageNode {
                                image: texture,
                                texture_atlas: Some(TextureAtlas {
                                    layout: asset_server.add(layout),
                                    index: 0, // Idle pose, first frame
                                }),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    }
                })),
            ));
        })),
    )
}

fn handle_layer_type_change(
    interaction_query: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<Button>)>,
    layer_button_query: Query<&LayerTypeButton>,
    player_level: Res<PlayerLevel>,
    mut state: ResMut<CharacterSelectState>,
    items_grid_query: Query<(Entity, &Children), With<ItemsGrid>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    for (interaction, child_of) in &interaction_query {
        if *interaction == Interaction::Pressed
            && let Ok(layer_button) = layer_button_query.get(child_of.parent())
        {
            state.current_layer_type = layer_button.0;

            // Rebuild items grid
            if let Ok((items_grid_entity, _)) = items_grid_query.single() {
                // Despawn existing items
                commands.entity(items_grid_entity).despawn_children();

                // Spawn new items for the selected layer type
                let available_items = state.current_layer_type.available_items();

                let mut available_items: Vec<_> = available_items.iter().map(Some).collect();

                // Require sufficient player level for certain clothing items
                let mut allowed_length = available_items.len();
                if state.current_layer_type == LayerType::Underclothes && player_level.0 < 3 {
                    allowed_length = 35;
                } else if state.current_layer_type == LayerType::Underclothes && player_level.0 < 4
                {
                    allowed_length = 40;
                } else if state.current_layer_type == LayerType::Underclothes && player_level.0 == 4
                {
                    allowed_length = 45;
                } else if state.current_layer_type == LayerType::Footwear && player_level.0 < 2 {
                    allowed_length = 6;
                }

                // Add empty item for all types except Body and Underclothes
                if state.current_layer_type != LayerType::Body
                    && state.current_layer_type != LayerType::Underclothes
                {
                    available_items.insert(0, None);
                }

                commands.entity(items_grid_entity).with_children(|parent| {
                    for index in 0..=allowed_length {
                        // reference playtime
                        let layer = available_items.get(index).and_then(|layer| *layer).cloned();
                        parent.spawn(item_button(layer, state.current_layer_type, &asset_server));
                    }
                });
            }
        }
    }
}

fn handle_item_selection(
    interaction_query: Query<(&Interaction, &ChildOf), (Changed<Interaction>, With<Button>)>,
    item_button_query: Query<&ItemButton>,
    mut state: ResMut<CharacterSelectState>,
) {
    for (interaction, child_of) in &interaction_query {
        if *interaction == Interaction::Pressed
            && let Ok(item_button) = item_button_query.get(child_of.parent())
        {
            if let Some(layer) = &item_button.layer {
                // Update or add the layer to current configuration
                if let Some(existing) = state
                    .current_layers
                    .layers
                    .iter_mut()
                    .find(|l| l.layer_type == item_button.layer_type)
                {
                    *existing = layer.clone();
                } else {
                    state.current_layers.layers.push(layer.clone());
                }
            } else {
                // Empty layer selected, remove matching layer
                state
                    .current_layers
                    .layers
                    .retain(|l| l.layer_type != item_button.layer_type);
            }
        }
    }
}

fn update_character_preview(
    state: Res<CharacterSelectState>,
    preview_query: Query<(Entity, &Children), With<CharacterPreview>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Only update if state changed
    if !state.is_changed() {
        return;
    }

    if let Ok((preview_entity, _)) = preview_query.single() {
        // Clear existing children
        commands.entity(preview_entity).despawn_children();

        // Spawn all layers
        let mut sorted_layers = state.current_layers.clone();
        sorted_layers.layers.sort_by_key(|l| l.layer_type);

        commands.entity(preview_entity).with_children(|parent| {
            for layer in sorted_layers.layers {
                if let Ok(path) = layer.texture_path() {
                    let texture = asset_server.load(path);
                    let layout = TextureAtlasLayout::from_grid(
                        UVec2::new(80, 64),
                        COLUMNS,
                        ROWS,
                        None,
                        None,
                    );

                    parent.spawn((
                        Name::new(format!("{:?} Preview Layer", layer.layer_type)),
                        ImageNode {
                            image: texture,
                            texture_atlas: Some(TextureAtlas {
                                layout: asset_server.add(layout),
                                index: 0, // Idle pose, first frame
                            }),
                            ..default()
                        },
                        UiTransform::from_scale(Vec2::splat(4.0)),
                        Pickable::IGNORE,
                        Node {
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                    ));
                }
            }
        });
    }
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Pause);
}

fn go_back(action_query: Query<&ActionState<Action>>, mut next_menu: ResMut<NextState<Menu>>) {
    if let Ok(action_state) = action_query.single() {
        if action_state.just_pressed(&Action::Menu) {
            next_menu.set(Menu::Pause);
        }
    }
}

fn confirm_selection(
    _: On<Pointer<Click>>,
    state: Res<CharacterSelectState>,
    mut next_screen: ResMut<NextState<Screen>>,
    mut next_menu: ResMut<NextState<Menu>>,
) {
    // TODO: Save the character configuration somewhere
    // For now, just proceed to gameplay
    info!("Character confirmed: {:?}", state.current_layers);
    next_menu.set(Menu::None);
    next_screen.set(Screen::Gameplay);
}

// Scroll handling systems

const LINE_HEIGHT: f32 = 100.;

/// UI scrolling event.
#[derive(EntityEvent, Debug)]
struct Scroll {
    entity: Entity,
    delta: Vec2,
}

/// Injects scroll events into the UI hierarchy.
fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= LINE_HEIGHT;
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.event_mut().delta;

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0. {
        let max = if delta.y > 0. {
            scroll_position.y >= max_offset.y
        } else {
            scroll_position.y <= 0.
        };

        if !max {
            scroll_position.y += delta.y;
            delta.y = 0.;
        }
    }
}

// Update scrollbar thumb color on hover/drag
fn update_scrollbar_thumb(
    mut q_thumb: Query<
        (&mut BackgroundColor, &Hovered, &CoreScrollbarDragState),
        (
            With<CoreScrollbarThumb>,
            Or<(Changed<Hovered>, Changed<CoreScrollbarDragState>)>,
        ),
    >,
) {
    for (mut thumb_bg, Hovered(is_hovering), drag) in q_thumb.iter_mut() {
        let color = if *is_hovering || drag.dragging {
            Srgba::hex("#FFFFFF").unwrap().into()
        } else {
            Srgba::hex("#888888").unwrap().into()
        };

        if thumb_bg.0 != color {
            thumb_bg.0 = color;
        }
    }
}
