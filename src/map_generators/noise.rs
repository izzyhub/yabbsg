use simdnoise::*;
use bevy::prelude::*;
use crate::tile_types::TileType;

pub fn noise_map(map_size: Vec2d, seed: i32) -> Mat2 {
    let noise = NoiseBuilder::fdm_2d(map_size.x, map_size.y)
    .with_seed(seed)
    .generate_scaled(0.0,1.0);

    let noise = noise.into_iter().map(|x| if x < 0.5 {TileType::Water} else {TileType::Grassland} ).collect();

    matrix::Conventional::from_vec((map_size.x, map_size.y), noise);
}