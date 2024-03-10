use rand_seeder::Seeder;

use bevyworld_lib::map_generators::{self, TileType};

use bevy::{
    diagnostic::LogDiagnosticsPlugin,
    ecs::query::QuerySingleError, math::uvec2, prelude::*,
    core_pipeline::core_2d::Camera2dBundle,
};

use bevy_inspector_egui::bevy_egui::{
    egui::{self},
    EguiContexts, EguiPlugin,
};

mod debug_plugin;
use debug_plugin::DebugPlugin;

use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundleManaged, MapIndexer};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Izzys cool 'game'".to_string(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource::<UiState>(UiState::new())
        .add_plugins(EguiPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FastTileMapPlugin)
        .add_systems(Update, ui_example_system)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Default, Resource)]
struct UiState {
    seed: String,
    //map_generator: MapGeneratorStrategies,
}

impl UiState {
    fn new() -> UiState {
        UiState {
            //map_generator: MapGeneratorStrategies::Voronoi,
            seed: "Initial Seed".to_string(),
        }
    }
}

fn ui_example_system(
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
    mut materials: ResMut<Assets<Map>>,
    maps: Query<&Handle<Map>>,
) {
    //let window = egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
    let _ = egui::Window::new("Map Generation").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::TextEdit::singleline(&mut ui_state.seed));
        //println!("response {:?}", response);
        if ui.add(egui::Button::new("Regenerate Map")).clicked() {
            match maps.get_single() {
                Ok(map_handle) => {
                    let map_size = Vec2::splat(2560.0);

                    println!("seed: {}", ui_state.seed);
                    let mut seeder = Seeder::from(ui_state.seed.clone());
                    let seed: [u8; 4] = seeder.make_seed();
                    let seed = u32::from_be_bytes(seed);

                    let params = MapGenParams {
                        map_size,
                        seed: seed as i32,
                    };

                    match materials.get_mut(map_handle) {
                        Some(map) => {
                            let mut map_indexer = map.indexer_mut();
                            gen_noise_map(params, &mut map_indexer);
                        }
                        None => {
                            println!("Failed to get a map from map handle");
                        }
                    }
                }
                Err(QuerySingleError::NoEntities(_)) => {
                    println!("No maps for some reason");
                }
                Err(QuerySingleError::MultipleEntities(_)) => {
                    println!("Why are there multiple maps");
                }
            };
        }
    });
}

struct MapGenParams {
    map_size: Vec2,
    seed: i32,
}

fn gen_noise_map(params: MapGenParams, map: &mut MapIndexer) {
    let tile_matrix = Box::new(map_generators::noise_map(
        params.map_size.x as usize,
        params.map_size.y as usize,
        params.seed,
    ));
    println!(
        "tile_matrix.shape: ({}, {})",
        tile_matrix.shape().0,
        tile_matrix.shape().1
    );

    for x in 0..params.map_size.x as u32 {
        for y in 0..params.map_size.y as u32 {
            let tile_type = tile_matrix.get((x as usize, y as usize));

            let tile_index = match tile_type {
                Some(tile_type) => tile_type.to_atlas_index(),
                None => {
                    println!("No tile at ({}, {})", x, y);
                    TileType::Water.to_atlas_index()
                }
            };

            map.set(x, y, tile_index);
        }
    }
  println!("map generation finished");
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    ui_state: Res<UiState>,
    mut materials: ResMut<Assets<Map>>,
) {
    let map_square = 1280;
    let map_size = Vec2::splat(2560.0);
    let tile_size = Vec2::splat(128.0);

    let texture = assets.load("tiles/multitiles.png");
    //let multitile_handle: Handle<Image> = assets.load("tiles/multitiles.png");

    let mut seeder = Seeder::from(ui_state.seed.clone());
    let seed: [u8; 4] = seeder.make_seed();
    let seed = u32::from_be_bytes(seed);

    let mut map = Map::builder(uvec2(map_square, map_square), texture, tile_size).build();

    let params = MapGenParams {
        map_size,
        seed: seed as i32,
    };

    let mut indexer = map.indexer_mut();
    gen_noise_map(params, &mut indexer);

    let mut camera = Camera2dBundle::default();
    //camera.projection.scale /= 2.0;
    //camera.projection.scale *= 4.0;
    camera.projection.scale *= 84.0;
    commands.spawn(camera);
    commands.spawn(MapBundleManaged::new(map, materials.as_mut()));
    //commands.spawn(Camera2dBundle::default());
}
