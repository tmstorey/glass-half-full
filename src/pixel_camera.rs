use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::window::WindowResized;

pub const GAME_HEIGHT: u32 = 320;
pub const MAX_WIDTH: u32 = 16_384;

#[derive(Component)]
pub struct PixelCamera;

#[derive(Component)]
pub struct MainCamera;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup_pixel_camera);
    app.add_systems(Update, fit_canvas);
}

fn setup_pixel_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let canvas_size = Extent3d {
        width: MAX_WIDTH,
        height: GAME_HEIGHT,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: canvas_size,
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

    image.resize(canvas_size);

    let image_handle = images.add(image);

    // Pixel camera that renders to the texture
    commands.spawn((
        Name::new("Pixel Camera"),
        Camera2d,
        Camera {
            target: RenderTarget::Image(image_handle.clone().into()),
            order: 0, // Render first
            ..default()
        },
        PixelCamera,
        RenderLayers::layer(0), // Render only entities on layer 0
    ));

    commands.spawn((
        Name::new("Canvas"),
        Sprite {
            image: image_handle.clone(),
            custom_size: Some(Vec2::new(MAX_WIDTH as f32, GAME_HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        RenderLayers::layer(1), // Render on a different layer
    ));

    // Main camera that displays the canvas
    commands.spawn((
        Name::new("Main Camera"),
        Camera2d,
        Camera {
            order: 1, // Render after the pixel camera
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        MainCamera,
        RenderLayers::layer(1), // Only render the canvas
    ));
}

/// Fit canvas to window on resize
fn fit_canvas(
    mut resize_messages: MessageReader<WindowResized>,
    mut projection: Single<&mut Projection, With<MainCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    for window_resized in resize_messages.read() {
        let (render_width, render_height) = calculate_render_size(
            window_resized.width,
            window_resized.height
        );
        let scale = calculate_scale(
            window_resized.width,
            window_resized.height,
            render_width,
            render_height
        );
        projection.scale = 1. / scale;
    }
}


/// Calculate the render target size based on window dimensions
/// Maintains 320 pixels height and adjusts width based on aspect ratio
fn calculate_render_size(window_width: f32, window_height: f32) -> (u32, u32) {
    let aspect_ratio = window_width / window_height;
    let height = GAME_HEIGHT;
    let width = (height as f32 * aspect_ratio).round() as u32;
    (width, height)
}

/// Calculate the scale factor to fit the render target into the window
fn calculate_scale(
    window_width: f32,
    window_height: f32,
    render_width: u32,
    render_height: u32,
) -> f32 {
    let scale_x = window_width / render_width as f32;
    let scale_y = window_height / render_height as f32;
    scale_x.min(scale_y)
}
