use super::level::BucketContent;
use super::{GameLevel, Season};
use crate::screens::Screen;
use bevy::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(Screen::Gameplay),
        (spawn_bucket_ui, spawn_level_info_ui),
    );
    app.add_systems(
        Update,
        (update_bucket_ui, update_level_info_ui).run_if(in_state(Screen::Gameplay)),
    );
}

/// Marker component for the bucket UI sprite
#[derive(Component)]
struct BucketUI;

/// Spawns the bucket UI indicator in the top-left corner
fn spawn_bucket_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Create the UI layout for the bucket indicator
    let texture = asset_server.load("images/ui/bucket-contents.epng");
    let layout = TextureAtlasLayout::from_grid(
        UVec2::splat(16),
        3, // 3 columns: empty, water, snow
        1, // 1 row
        None,
        None,
    );

    commands
        .spawn((
            Name::new("Bucket UI Root"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                width: Val::Px(48.0),
                height: Val::Px(48.0),
                ..default()
            },
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Bucket Icon"),
                BucketUI,
                ImageNode {
                    image: texture,
                    texture_atlas: Some(TextureAtlas {
                        layout: asset_server.add(layout),
                        index: 0, // Start with empty bucket
                    }),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
            ));
        });
}

/// Updates the bucket UI sprite based on the current bucket content
fn update_bucket_ui(
    bucket_content: Res<BucketContent>,
    mut query: Query<&mut ImageNode, With<BucketUI>>,
) {
    if !bucket_content.is_changed() {
        return;
    }

    for mut image in query.iter_mut() {
        if let Some(ref mut atlas) = image.texture_atlas {
            atlas.index = match *bucket_content {
                BucketContent::Empty => 0,
                BucketContent::Water => 1,
                BucketContent::Snow => 2,
            };
        }
    }
}

/// Marker component for the level info text
#[derive(Component)]
struct LevelInfoText;

/// Spawns the level info UI in the top-right corner
fn spawn_level_info_ui(mut commands: Commands, season: Res<Season>, game_level: Res<GameLevel>) {
    commands
        .spawn((
            Name::new("Level Info UI Root"),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(64.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..default()
            },
            DespawnOnExit(Screen::Gameplay),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Level Info Text"),
                LevelInfoText,
                Text::new(format!("{}\nLevel {}", *season, game_level.0)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ));
        });
}

/// Updates the level info text when season or level changes
fn update_level_info_ui(
    season: Res<Season>,
    game_level: Res<GameLevel>,
    mut query: Query<&mut Text, With<LevelInfoText>>,
) {
    if !season.is_changed() && !game_level.is_changed() {
        return;
    }

    for mut text in query.iter_mut() {
        **text = format!("{}\nLevel {}", *season, game_level.0);
    }
}
