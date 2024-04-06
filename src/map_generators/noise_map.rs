use crate::map_generators::tile_types::TileType;
use crate::map_generators::tile::Tile;
//use fastnoise_lite::*;
use nalgebra::DMatrix;
//use simdnoise::*;
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex, RidgedMulti, Seedable};
use lerp::Lerp;
use rand_seeder::Seeder;
use crate::math_helpers::create_new_seed32;


pub struct NoiseMapOptions {
    map_width: usize,
    map_height: usize,
    seed: u32,
}

impl NoiseMapOptions {
    pub fn new(map_width: usize, map_height: usize, seed: u32) -> NoiseMapOptions {
        NoiseMapOptions {
            map_width,
            map_height,
            seed,
        }
    }
}

impl Default for NoiseMapOptions {
    fn default() -> Self {
        let mut seeder = Seeder::from("Initial Seed");
        let seed = create_new_seed32(&mut seeder);
        NoiseMapOptions {
            map_width: 1920,
            map_height: 1080,
            seed: seed,
        }
    }
}

pub fn distance_from_center(x: f64, y: f64, width: f64, height: f64) -> f64 {
    let nx = 2.0*(x / width) - 1.0;
    let ny = 2.0*(y / height) - 1.0;
    1.0 - (1.0 - nx.powi(2)) * (1.0 - ny.powi(2))
}

pub fn noise_map(options: NoiseMapOptions) -> DMatrix<Tile> {
    println!("map_size_x: {}", options.map_width);
    println!("map_size_y: {}", options.map_height);

    //let noise = NoiseBuilder::cellular_2d(map_size_x  as usize, map_size_y as usize)
    /* 
    let noise = NoiseBuilder::fbm_2d(map_size_x  as usize, map_size_y as usize)
    //let noise = NoiseBuilder::gradient_2d(map_size_x  as usize, map_size_y as usize)
    .with_freq(0.08)
    .with_octaves(30)
    //.with_gain(2.0)
    .with_seed(seed)
    .with_lacunarity(0.5)
    .generate_scaled(-1.0, 1.0);
    */
    //let noise = OpenSimplex::new(seed as u32);
    let noise: Fbm<OpenSimplex> = Fbm::new(options.seed);
    let noise = noise.set_octaves(80);
    let noise = noise.set_frequency(0.002);

    DMatrix::from_fn(options.map_height, options.map_width, |row, column| {
        //println!("row: {row}");
        //println!("column: {column}");
        //println!("row_x: {column_x}");
        //println!("column_y: {row_y}");
        let distance = distance_from_center(column as f64, row as f64, options.map_width as f64, options.map_height as f64);
        //println!("distance: {distance}");
        //let noise_val = noise.get([row as f64, column as f64]);
        let noise_val = noise.get([column as f64, row as f64]);
        //println!("noise_val: {noise_val}");
        let new_noise_val = noise_val.lerp(1.0 - distance, 0.50);
        //println!("lerped_noise_val: {new_noise_val}");


        let mut tile_type = TileType::Grassland;
        if new_noise_val < 0.0 {
            tile_type = TileType::Water;
        }

        Tile::new(tile_type, new_noise_val)
    })
}

#[cfg(test)]
mod tests {
    use super::distance_from_center;
    use lerp::Lerp;

    #[test]
    fn test_lerp() {
        let lerped = 0.0.lerp(1.0 - 1.0, 0.50);
        assert_eq!(lerped, 0.0);
    }
    #[test]
    fn test_distance_from_center() {
        let width = 100.0;
        let height = 100.0;

        // Top left corner
        let distance = distance_from_center(0.0, 0.0, width, height);
        assert_eq!(distance, 1.0);

        // Top right corner
        let distance = distance_from_center(width, 0.0, width, height);
        assert_eq!(distance, 1.0);

        // center
        let distance = distance_from_center(width / 2.0, height / 2.0, width, height);
        assert_eq!(distance, 0.0);

        // Bottom left corner
        let distance = distance_from_center(0.0, height, width, height);
        assert_eq!(distance, 1.0);

        // bottom right corner
        let distance = distance_from_center(width, height, width, height);
        assert_eq!(distance, 1.0);

        let distance = distance_from_center(1920., 0., 1920., 1080.);
        assert_eq!(distance, 1.0);
    }
}