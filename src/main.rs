use core::fmt;

use bevyworld_lib::map_generators;

use bevy::{
    core::Zeroable, diagnostic::LogDiagnosticsPlugin, prelude::*
};
use bevy_egui::{egui::{self, color_picker::color_picker_color32}, EguiContexts, EguiPlugin};

fn main() {
    App::new()
    .add_plugins((DefaultPlugins, LogDiagnosticsPlugin::default()))
    .insert_resource::<UiState>(UiState::new())
    .add_plugins(EguiPlugin)
    .add_systems(Update, ui_example_system)
    .add_systems(Startup, setup)
    .run();
}

#[derive(PartialEq, Debug, Default)]
enum MapGeneratorStrategies {
    #[default]
    WaveFunctionCollapse,
    PerlinNoise
}

impl fmt::Display for MapGeneratorStrategies {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MapGeneratorStrategies::WaveFunctionCollapse => write!(f, "Wave Function Collapse"),
            MapGeneratorStrategies::PerlinNoise => write!(f, "Perlin Noise"),
        }
    }
}

#[derive(Default, Resource)]
struct UiState {
    map_generator: MapGeneratorStrategies,
}

impl UiState {
    fn new() -> UiState {
        UiState {
            map_generator: MapGeneratorStrategies::WaveFunctionCollapse,
        }
    }
}

fn ui_example_system(mut ui_state: ResMut<UiState>, mut contexts: EguiContexts) {
    egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
        ui.label("world");
        egui::ComboBox::from_label("map generator strategies")
            .selected_text(format!("{}", ui_state.map_generator))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut ui_state.map_generator, MapGeneratorStrategies::WaveFunctionCollapse, "Wave Function Collapse");
                ui.selectable_value(&mut ui_state.map_generator, MapGeneratorStrategies::PerlinNoise, "Perlin Noise");
        });
    });
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let tile_size = Vec2::splat(32.0);
    let map_size = Vec2::splat(1280.0);
    let half_x = (map_size.x / 2.0) as i32;
    let half_y = (map_size.y / 2.0) as i32;

    let water_handle = assets.load("tiles/water2_0.png");

    commands.spawn(Camera2dBundle::default());
    let mut sprites = vec![];

    for y in -half_y..half_y {
        for x in -half_x..half_x {
            let position = Vec2::new(x as f32, y as f32);
            let translation = (position * tile_size).extend(0.0);
            let rotation  = Quat::zeroed();
            let scale = Vec3::new(1.0, 1.0, 1.0);

            sprites.push(SpriteBundle {
                texture: water_handle.clone(),
                transform: Transform {
                    translation,
                    rotation,
                    scale,
                },
                sprite: Sprite {
                    custom_size: Some(tile_size),
                    color: Color::WHITE,
                    ..default()
                },
            ..default()
            });
        }
    }
    commands.spawn_batch(sprites);

}