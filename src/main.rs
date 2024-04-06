use noise::math;
use rand_seeder::Seeder;

use bevyworld_lib::{map_generators::{self, NoiseMapOptions, TileType}, math_helpers};

use bevy::{
    core_pipeline::core_2d::Camera2dBundle, diagnostic::LogDiagnosticsPlugin,
    ecs::{query::QuerySingleError, world}, math::uvec2, prelude::*, window::PrimaryWindow,
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
                        resolution: (1920., 1080.).into(),
                        ..default()
                    }),
                    ..default()
                }),
            LogDiagnosticsPlugin::default(),
        ))
        .insert_resource::<UiState>(UiState::new())
        .init_resource::<WorldCoords>()
        .add_plugins(EguiPlugin)
        .add_plugins(DebugPlugin)
        .add_plugins(FastTileMapPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Startup, setup_world_coords)
        .add_systems(Update, ui_system)
        .add_systems(Update, cursor_system)
        .run();
}


/// We will store the world position of the mouse cursor here.
#[derive(Resource, Default)]
struct WorldCoords(Vec2);
#[derive(Component)]
struct WorldCoordsText;
/// Used to help identify our main camera
#[derive(Component)]
struct WindowCoordsText;
#[derive(Component)]
struct MainCamera;


fn setup_world_coords(
    mut commands: Commands,
) {
    // check if the cursor is inside the window and get its position
    // then, ask bevy to convert into world coordinates, and truncate to discard Z
    let font_size = 30.0;
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "world mouse position: ",
                TextStyle {
                    font_size: font_size,
                    ..default()
                }
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    ..default()
                  }
            ),
        ])
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(75.0),
            right: Val::Px(5.0),
            ..default()
        }),
        WorldCoordsText
    ));

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "window mouse position: ",
                TextStyle {
                    font_size: font_size,
                    ..default()
                }
            ),
            TextSection::from_style(
                TextStyle {
                    font_size: 30.0,
                    ..default()
                  }
            ),
        ])
        .with_text_justify(JustifyText::Center)
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
            ..default()
        }),
        WindowCoordsText
    ));

}

fn cursor_system(
    mut mycoords: ResMut<WorldCoords>,
    // query to get the window (so we can read the current cursor position)
    mut q_world_text: Query<&mut Text, (With<WorldCoordsText>, Without<WindowCoordsText>)>,
    mut q_window_text: Query<&mut Text, (With<WindowCoordsText>, Without<WorldCoordsText>)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so Query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // There is only one primary window, so we can similarly get it from the query:
    let window = q_window.single();

    let (world_text, window_text) = match window.cursor_position() {
        Some(cursor_position) =>
        {
            let world_position = camera.viewport_to_world(camera_transform, cursor_position)
            .map(|ray| ray.origin.truncate());
            match world_position {
                Some(world_position) => {
                    mycoords.0 = world_position;
                    (format!("({}, {})", world_position.x, world_position.y), 
                    format!("({}, {})", cursor_position.x, cursor_position.y))
                },
                None => {
                    ("N/A".to_string(), 
                    format!("({}, {})", cursor_position.x, cursor_position.y))
                }
            }
            //text.sections[1].value = format!("({}, {})", world_position.x, world_position.y);
        }
        None => {
            ("N/A".to_string(), "N/A".to_string())

        }

    };

    for mut text in &mut q_world_text {
        text.sections[1].value = world_text.clone();
    }
    for mut text in &mut q_window_text {
        text.sections[1].value = window_text.clone();
    }
}

enum MapGeneratorStrategies {
    NoiseMap(NoiseMapOptions),
}

impl Default for MapGeneratorStrategies {
    fn default() -> Self {
        MapGeneratorStrategies::NoiseMap(NoiseMapOptions::default())
    }
}

#[derive(Default, Resource)]
struct UiState {
    seed: String,
    voronoi_cell_count: usize,
    map_generator: MapGeneratorStrategies,
}

impl UiState {
    fn new() -> UiState {
        UiState {
            seed: "Initial Seed".to_string(),
            voronoi_cell_count: 120,
            map_generator: MapGeneratorStrategies::default(),
        }
    }
}

fn ui_system(
    mut ui_state: ResMut<UiState>,
    mut contexts: EguiContexts,
    mut materials: ResMut<Assets<Map>>,
    maps: Query<&Handle<Map>>,
) {
    //let window = egui::Window::new("Hello").show(contexts.ctx_mut(), |ui| {
    let _ = egui::Window::new("Map Generation").show(contexts.ctx_mut(), |ui| {
        ui.add(egui::TextEdit::singleline(&mut ui_state.seed));
        ui.add(egui::DragValue::new(&mut ui_state.voronoi_cell_count).speed(1));
        //println!("response {:?}", response);
        if ui.add(egui::Button::new("Regenerate Map")).clicked() {
            match maps.get_single() {
                Ok(map_handle) => {
                    let width = 1920;
                    let height = 1080;

                    println!("seed: {}", ui_state.seed);
                    let mut seeder = Seeder::from(ui_state.seed.clone());
                    let seed = math_helpers::create_new_seed32(& mut seeder);


                    match materials.get_mut(map_handle) {
                        Some(map) => {
                            let options = NoiseMapOptions::new(width, height, seed);
                            let mut map_indexer = map.indexer_mut();
                            gen_noise_map(MapGeneratorStrategies::NoiseMap(options), &mut map_indexer);
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


fn gen_noise_map(map_generator: MapGeneratorStrategies, map: &mut MapIndexer) {
    let tile_matrix = match map_generator {
        MapGeneratorStrategies::NoiseMap(options) => {
            map_generators::noise_map(options)
        }
    };
    /* 
    let tile_matrix = match map_generators::voronoi_continents(
        params.map_size.x as usize,
        params.map_size.y as usize,
        &mut params.seeder,
        params.voronoi_cell_count,
    ) {
        Ok(matrix) => matrix,
        Err(error) => {
            println!("error: {error:?}");
            return
        }
    };
*/
    println!(
        "tile_matrix.shape: ({}, {})",
        tile_matrix.shape().0,
        tile_matrix.shape().1
    );

    for x in 0..tile_matrix.nrows() {
        for y in 0..tile_matrix.ncols() {
            let tile_type = tile_matrix.get((x, y));

            let tile_index = match tile_type {
                //Some(tile_type) => tile_type.terrain().to_atlas_index(),
                Some(tile_type) => tile_type.terrain().to_atlas_index(),
                None => {
                    println!("No tile at ({}, {})", x, y);
                    TileType::Water.to_atlas_index()
                }
            };

            map.set(x as u32, y as u32, tile_index);
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
    let map_width = 1920;
    let map_height = 1080;
    //let map_size = Vec2::new(map_height, map_width);
    let tile_size = Vec2::splat(128.0);

    let texture = assets.load("tiles/multitiles.png");
    //let multitile_handle: Handle<Image> = assets.load("tiles/multitiles.png");

    let mut seeder = Seeder::from(ui_state.seed.clone());
    let seed = math_helpers::create_new_seed32(&mut seeder);

    let mut map = Map::builder(uvec2(map_width as u32, map_height as u32), texture, tile_size).build();

    let mut indexer = map.indexer_mut();
    let options = NoiseMapOptions::new(map_width, map_height, seed);
    gen_noise_map(MapGeneratorStrategies::NoiseMap(options), &mut indexer);

    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 115.0;
    //camera.projection.scale /= 2.0;
    //camera.projection.scale *= 4.0;
    //camera.projection.scale *= 1932.0;
    //camera.projection.scale *= 250.0;
    commands.spawn((camera, MainCamera));
    commands.spawn(MapBundleManaged::new(map, materials.as_mut()));
    //commands.spawn(Camera2dBundle::default());
}
