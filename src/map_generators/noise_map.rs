use crate::map_generators::tile_types::TileType;
use fastnoise_lite::*;
use nalgebra::DMatrix;

pub fn noise_map(map_size_x: usize, map_size_y: usize, seed: i32) -> DMatrix<TileType> {
    println!("map_size_x: {map_size_x}");
    println!("map_size_y: {map_size_y}");

    let mut noise = FastNoiseLite::with_seed(seed);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(2));
    //noise.set_fractal_octaves(Some(20));
    //noise.set_frequency(Some(8.0));

    DMatrix::from_fn(map_size_x, map_size_y, |row, column| {
        let noise_val = noise.get_noise_2d(row as f32, column as f32);
        if noise_val < 0.25 {
            TileType::Water
        } else {
            TileType::Grassland
        }
    })
}
