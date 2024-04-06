//use core::num;

use crate::map_generators::tile::Tile;
use crate::map_generators::tile_types::TileType;
use crate::math_helpers::*;
use nalgebra::DMatrix;
use rand::prelude::*;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

use voronoice::*;

use std::collections::HashSet;

use fastnoise_lite::*;

use thiserror::Error;
use rand::distributions::WeightedError;

#[derive(Error, Debug)]
pub enum VoronoiContinentError {
    #[error("Failed to generate a diagram")]
    DiagramCreationError,
    #[error(transparent)]
    DiagramChooseError(#[from] WeightedError),
}


pub struct ContinentOptions {
    total_area: usize,
    land_area_percentage: f64,
    num_continents: usize,
    water_padding_percentage: f64,
}

impl ContinentOptions {
    pub fn new(total_area: usize, land_area_percentage: f64, num_continents: usize, water_padding_percentage: f64) -> ContinentOptions {
        ContinentOptions {
            total_area,
            land_area_percentage,
            num_continents,
            water_padding_percentage: (100.0 - water_padding_percentage) / 100.0,
        }
    }
}

pub struct Continents {
    pub all_continent_cells: HashSet<usize>,
    pub continents: Vec<HashSet<usize>>,
    pub initial_cells: HashSet<usize>,
}
impl Continents {
    fn new(all_continent_cells: HashSet<usize>, continents: Vec<HashSet<usize>>, initial_cells: HashSet<usize>) -> Continents {
        Continents {
            all_continent_cells,
            continents,
            initial_cells,
        }
    }
}

pub fn make_continent_cells(diagram: &Voronoi, options: &ContinentOptions, rng: &mut Pcg64) -> Result<Continents, VoronoiContinentError> {
    let land_box = land_box(&diagram.bounding_box(), options.water_padding_percentage);
    
    let all_cells: Vec<VoronoiCell> = diagram.iter_cells().collect();
    let initial_continent_cells = all_cells.choose_multiple_weighted(rng, options.num_continents, |cell| {
        if is_cell_in_box(cell, &land_box) {
            1.0
        } else {
            0.0
        }
    }).map_err(VoronoiContinentError::DiagramChooseError)?;
    let initial_continent_cells: HashSet<usize> = initial_continent_cells.map(|cell| cell.site()).collect();

    let initial_cells: HashSet<usize> = initial_continent_cells.iter().copied().collect();

    let mut used_cells: HashSet<usize> = initial_continent_cells.into_iter().collect();

    let mut continents: Vec<HashSet<usize>> = used_cells
        .iter()
        .map(|cell| HashSet::from([*cell]))
        .collect();

    let mut land_area: f64 = continents
        .iter()
        .map(|continent_cells| {
            continent_cells
                .iter()
                .map(|cell_index| shoelace_area_of_cell(diagram.cell(*cell_index)))
                .sum::<f64>()
        })
        .sum();

    let mut land_area_percentage: f64 = (land_area / options.total_area as f64) * 100.0;

    while land_area_percentage <= options.land_area_percentage {
        let continent_cell_set = continents.choose_mut(rng);
        match continent_cell_set {
            Some(cell_set) => {
                let cell_index = cell_set.iter().choose(rng);
                match cell_index {
                    Some(index) => {
                        let cell = diagram.cell(*index);
                        let mut neighbors: Vec<usize> = cell.iter_neighbors().collect();
                        neighbors.shuffle(rng);

                        for neighbor in neighbors {
                            let neighbor_cell = diagram.cell(neighbor);
                            if !used_cells.contains(&neighbor) && land_box.is_inside(neighbor_cell.site_position()){
                                used_cells.insert(neighbor);
                                cell_set.insert(neighbor);
                                let cell_area =
                                    shoelace_area_of_cell(diagram.cell(neighbor));
                                land_area += cell_area;
                                land_area_percentage = (land_area / options.total_area as f64) * 100.0;
                                break;
                            }
                        }
                    }
                    None => {
                        println!("somehow we got an empty continent cell")
                    }
                }
            }
            None => {
                println!("somehow we got an empty continent set")
            }
        }
    }

    Ok(Continents::new(used_cells, continents, initial_cells))

}

pub fn voronoi_continents(
    map_size_x: usize,
    map_size_y: usize,
    seeder: &mut Seeder,
    cell_count: usize,
) -> Result<Box<DMatrix<Tile>>, VoronoiContinentError> {
    let half_x = map_size_x as f64 / 2.0;
    let half_y = map_size_y as f64 / 2.0;
    let mut rng: Pcg64 = seeder.make_rng();

    let x_range = rand::distributions::Uniform::new(-half_x, half_x);
    let y_range = rand::distributions::Uniform::new(-half_y, half_y);
    let sites: Vec<Point> = (0..cell_count)
        .map(|_| Point {
            x: rng.sample(x_range),
            y: rng.sample(y_range),
        })
        .collect();
 
    let voronoi_diagram = VoronoiBuilder::default()
        .set_sites(sites)
        .set_bounding_box(BoundingBox::new_centered(
            map_size_x as f64,
            map_size_y as f64,
        ))
        .set_lloyd_relaxation_iterations(30)
        .build()
        .ok_or(VoronoiContinentError::DiagramCreationError)?;

    let total_area = map_size_x * map_size_y;
    let ideal_land_area_percentage_lower_bound = 29.0;
    let num_continents: usize = 8;

    let options = ContinentOptions::new(total_area, ideal_land_area_percentage_lower_bound, num_continents, 7.0);
    let continents = make_continent_cells(&voronoi_diagram, &options, &mut rng)?;

    let noise_x_seed: [u8; 4] = seeder.make_seed();
    let noise_x_seed = u32::from_be_bytes(noise_x_seed);

    let mut noise_x = FastNoiseLite::with_seed(noise_x_seed as i32);
    noise_x.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise_x.set_fractal_type(Some(FractalType::FBm));
    noise_x.set_fractal_octaves(Some(2));

    let noise_y_seed: [u8; 4] = seeder.make_seed();
    let noise_y_seed = u32::from_be_bytes(noise_y_seed);
    let mut noise_y = FastNoiseLite::with_seed(noise_y_seed as i32);
    noise_y.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise_y.set_fractal_type(Some(FractalType::FBm));
    noise_y.set_fractal_octaves(Some(2));


    Ok(Box::new(DMatrix::from_fn(map_size_x, map_size_y, |row, column| {
        let row_x: f64 = (row as f64) - half_x;
        let column_y: f64 = (column as f64) - half_y;

        let noise_x_value = noise_x.get_noise_2d(row_x as f32, column_y as f32);
        let noise_y_value = noise_y.get_noise_2d(row_x as f32, column_y as f32);
        let point_x = row_x + noise_x_value as f64;
        let point_y = column_y + noise_y_value as f64;

        let point = Point {
            x: point_x,
            y: point_y,
        };
        let closest_index = closest_cell(&point, &voronoi_diagram);
        let closest_cell = voronoi_diagram.cell(closest_index);

        if continents.all_continent_cells.contains(&closest_cell.site()) {
            Tile::new(TileType::Grassland, 0.0)
        } else {
            Tile::new(TileType::Water, 0.0)
        }
    })))
}

#[cfg(test)]
mod tests {
}
