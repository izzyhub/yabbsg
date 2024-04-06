use crate::map_generators::tile::Tile;
use crate::map_generators::tile_types::TileType;
use crate::math_helpers::*;
use bevy::ecs::component::TickCells;
use bevy_inspector_egui::egui::special_emojis::GIT;
use nalgebra::DMatrix;
use rand::prelude::*;
use rand_pcg::Pcg64Mcg;
use rand_seeder::Seeder;

use voronoice::*;
use std::collections::HashSet;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use fastnoise_lite::*;

use thiserror::Error;
use rand::distributions::WeightedError;

#[derive(Error, Debug)]
pub enum VoronoiError {
    #[error("Failed to generate a diagram")]
    DiagramCreationError,
    #[error(transparent)]
    DiagramChooseError(#[from] WeightedError),
}

#[derive(Debug)]
pub struct MapCell {
    pub cell_index: usize,
    pub height: f64,
}

impl MapCell {
    pub fn new (cell_index: usize, height: f64) -> MapCell {
        MapCell {
            cell_index,
            height,
        }
    }
}

pub struct HeightMapOptions {
    max_height: f64,
    height_step: f64,
    variance: Option<f64>,
    water_padding_percentage: f64,
    initial_sites: usize,
}

impl HeightMapOptions {
    pub fn new(max_height: f64, height_step: f64, variance: Option<f64>, water_padding_percentage: f64, initial_sites: usize) -> HeightMapOptions {
        HeightMapOptions {
            max_height,
            height_step,
            variance,
            water_padding_percentage,
            initial_sites,
        }
    }
}




pub fn diagram_to_heightmap(diagram: &Voronoi, rng: &mut Pcg64Mcg, options: HeightMapOptions) -> Result<HashMap<usize, MapCell>, VoronoiError> {
    let land_box = land_box(&diagram.bounding_box(), options.water_padding_percentage);
    let cells: Vec<VoronoiCell> = diagram.iter_cells().collect();
    let initial_cells = cells.choose_multiple_weighted(rng, options.initial_sites,  |cell| {
        if is_cell_in_box(cell, &land_box) {
            1.0
        }
        else {
            0.0
        }
    }).map_err(VoronoiError::DiagramChooseError)?;
    
    let total_sites = diagram.sites().len();
    let mut cells = HashMap::with_capacity(total_sites);
    let mut used_cells = HashSet::with_capacity(total_sites);

    let mut cell_queue = VecDeque::with_capacity(total_sites);
    let mut current_height = options.max_height;
    for cell in initial_cells {
        let cell_index = cell.site();
        used_cells.insert(cell_index);
        cells.insert(cell_index, MapCell::new(cell_index, options.max_height));
        cell_queue.extend(cell.iter_neighbors());
    }
    
    while used_cells.len() < total_sites {
        while let Some(cell_index) = cell_queue.pop_front() {
            let cell = diagram.cell(cell_index);
            let in_water_buffer= !is_cell_in_box(&cell, &land_box);
            if !in_water_buffer {
                current_height = current_height * options.height_step;
            }

            let height_variance = if !in_water_buffer {
                options.variance.map_or(1.0, |variance| rng.gen::<f64>() * variance + 1.1  - variance)
            } else {
                0.0
            };

            let new_height = current_height * height_variance;
            match cells.get(&cell_index) {
                Some(map_cell) => {
                    if map_cell.height < new_height {
                        cells.remove(&cell_index);
                        cells.insert(cell_index, MapCell::new(cell_index, new_height));
                    }
                }
                None => {
                    cells.insert(cell_index, MapCell::new(cell_index, new_height));
                    used_cells.insert(cell_index);
                }
            }
            for neighbor in cell.iter_neighbors() {
                if !used_cells.contains(&neighbor) {
                    cell_queue.push_back(neighbor);
                }
            }
        }
    }

    Ok(cells)
}

pub fn voronoi_heightmap(
    map_size_x: u64,
    map_size_y: u64,
    seeder: &mut Seeder,
    cell_count: u64,
    options: HeightMapOptions,
) -> Result<DMatrix<Tile>, VoronoiError> {
    let half_x = map_size_x as f64 / 2.0;
    let half_y = map_size_y as f64 / 2.0;
    let mut rng: Pcg64Mcg = seeder.make_rng();

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
        .ok_or(VoronoiError::DiagramCreationError)?;


    //let cell_indexes = voronoi_diagram.sites().choose_multiple(rng, 2);
    todo!()

}