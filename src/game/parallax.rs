
use bevy::prelude::*;
use bevy::image::{ImageLoaderSettings, ImageSampler};
use strum::Display;


#[derive(Clone, Copy, Display, Reflect)]
pub enum Season {
    Summer,
    Autumn,
    Winter,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ParallaxLayer {
    pub season: Season,
    pub layer: f32,
    /// Movement relative to camera (0.0 = static, 1.0 = moves with camera)
    pub scroll_factor: f32,
}

pub fn parallax_background(
    season: Season,
    asset_server: Res<AssetServer>,
) -> impl Bundle {
    let mut children = vec![];
    let scroll_factors = vec![0.1, 0.5, 0.7, 0.95, 1.0];
    for layer in 1..=5 {
        let z = layer as f32;
        let scroll_factor = scroll_factors[layer - 1];
        let name = format!("{}-{}", season, layer).to_lowercase();
        let image = asset_server.load_with_settings(
            format!("images/bg/{}.epng", name),
            |settings: &mut ImageLoaderSettings| {
                settings.sampler = ImageSampler::nearest();
            },
        );
        children.push((
            ParallaxLayer { season, layer: z, scroll_factor },
            Name::new(name),
            Sprite{
                image,
                custom_size: Some(Vec2::new(16_384.,346.)),
                image_mode: SpriteImageMode::Tiled {
                    tile_x: true,
                    tile_y: false,
                    stretch_value: 1.0,
                },
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, -z)
        ));
    }
    (
        Name::new(format!("Parallax ({})", season)),
        Transform::default(),
        Visibility::default(),
        Children::spawn((
            SpawnIter(children.into_iter()),
        ))
    )
}

pub fn scroll_parallax(
    time: Res<Time>,
    mut query: Query<(&ParallaxLayer, &mut Transform)>,
) {
    let base_speed = 50.0;

    for (layer, mut transform) in &mut query {
        let speed = base_speed * (1. - layer.scroll_factor);
        transform.translation.x -= speed * time.delta_secs();
        transform.translation.x = transform.translation.x % 1024.0;
    }
}
