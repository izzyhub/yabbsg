//use core::num;

use crate::map_generators::tile_types::TileType;
use crate::map_generators::tile::Tile;
use crate::math_helpers::*;
//use bevy::log::tracing_subscriber::filter::targets::Iter;
//use bevy_inspector_egui::egui::was_tooltip_open_last_frame;
use rand::prelude::*;
use rand_seeder::Seeder;
use rand_pcg::Pcg64;
use nalgebra::DMatrix;

use voronoice::*;

use std::collections::HashSet;

use fastnoise_lite::*;

fn closest_cell(point: &Point, diagram: &Voronoi) -> usize {
    let cell_count = diagram.sites().len();
    let mut closest_cell = diagram.cell(0);
    let mut min_distance = f64::MAX;
    for index in 0..cell_count {
        let cell = diagram.cell(index);
        let cell_distance = distance(&point, cell.site_position());
        if cell_distance  < min_distance {
            min_distance = cell_distance;
            closest_cell = cell;
        }
    }

    closest_cell.site()
}

pub fn voronoi_continents(map_size_x: usize, map_size_y: usize, seeder: &mut Seeder, mut cell_count: usize) -> DMatrix<Tile> {
    println!("generating continents");
    let half_x = map_size_x as f64 / 2.0;
    let half_y = map_size_y as f64 / 2.0;
    let mut rng: Pcg64 = seeder.make_rng();
    //let mut sites = Vec::with_capacity(cell_count);

    let x_range = rand::distributions::Uniform::new(-half_x, half_x);
    let y_range = rand::distributions::Uniform::new(-half_y, half_y);
    let sites: Vec<Point> = (0..cell_count).map(|_| Point {
        x: rng.sample(x_range),
        y: rng.sample(y_range)
    }).collect();

    //for _ in 0..cell_count {
        //let x = rng.gen_range(-half_x..=half_x as f64);
        //let y = rng.gen_range(-half_y..=half_y as f64);
        //sites.push(Point {
            //x, y
        //})
    //}
    println!("initial sites: {sites:?}");
    println!("sites.len: {}", sites.len());
    let copied_sites: Vec<Point> = sites.iter().map(|site| site.clone()).collect();

    let voronoi_diagram = VoronoiBuilder::default()
      .set_sites(sites)
      .set_bounding_box(BoundingBox::new_centered(map_size_x as f64, map_size_y as f64))
      .set_lloyd_relaxation_iterations(30)
      .build().expect("Failed to build a voronoi diagram");

    println!("map_size_x: {map_size_x}");
    println!("map_size_y: {map_size_y}");
    let bounding_box = voronoi_diagram.bounding_box();
    for (index, corner) in bounding_box.corners().iter().enumerate() {
        println!("corner{index} {corner:?}");
    }
    for site in copied_sites {
        println!("{site:?} inside bounding box: {}", bounding_box.is_inside(&site));
    }

    println!("original cell_count: {}", cell_count);
    cell_count = voronoi_diagram.sites().len();
    println!("actual cell_count: {}", cell_count);

    let total_area = map_size_x * map_size_y;
    //let ideal_land_area_percentage_lower_bound = 29.0;
    let ideal_land_area_percentage_lower_bound = 2.0;
    let num_continents: usize = rng.gen_range(4..=9);

    println!("generated continents");
    let initial_continent_cells = voronoi_diagram.iter_cells().map(|cell| cell.site()).choose_multiple(&mut rng, num_continents);
    let mut used_cells: HashSet<usize> = initial_continent_cells.into_iter().collect();
    println!("initial_continents: {used_cells:?}");

    let mut continents: Vec<HashSet<usize>> = used_cells.iter().map(|cell| {
        HashSet::from([cell.clone()])
    }).collect();
    // we want to iterate over the segments until we assign enough of them to be land to be roughly earth analogous


    let mut land_area: f64 = continents.iter().map(|continent_cells| {
        continent_cells.iter().map(|cell_index| {
            shoelace_area_of_cell(voronoi_diagram.cell(*cell_index))
        }).sum::<f64>()
    }).sum();

    let mut land_area_percentage: f64 = (land_area / total_area as f64) * 100.0;

    println!("total area: {total_area}");
    println!("initial land area: {land_area}");
    println!("initial land area percentage: {land_area_percentage}");

    while land_area_percentage <= ideal_land_area_percentage_lower_bound {
        println!("total area: {total_area}");
        println!("land area: {land_area}");
        println!("land area precentage: {land_area_percentage}");
        let continent_cell_set = continents.choose_mut(&mut rng);
        match continent_cell_set {
            Some(cell_set) => {
                let cell_index = cell_set.iter().choose(&mut rng);
                match cell_index {
                    Some(index) => {
                        println!("index: {index}");
                        let cell = voronoi_diagram.cell(*index);
                        let mut neighbors: Vec<usize> = cell.iter_neighbors().collect();
                        println!("neighbors: {neighbors:?}");
                        neighbors.shuffle(&mut rng);
                        println!("neighbors: {neighbors:?}");
                        println!("used_cells: {used_cells:?}");

                        for neighbor in neighbors {
                            println!("neighbor: {neighbor}");
                            if !used_cells.contains(&neighbor) {
                                println!("adding neighbor to continents");
                                used_cells.insert(neighbor);
                                println!("used_cells: {used_cells:?}");
                                cell_set.insert(neighbor);
                                let cell_area = shoelace_area_of_cell(voronoi_diagram.cell(neighbor));
                                println!("neighbor area: {cell_area}");
                                land_area += cell_area;
                                println!("new land area: {land_area}");
                                land_area_percentage = (land_area / total_area as f64) * 100.0;
                                println!("new land area percentage: {land_area_percentage}");
                                break
                            }
                        }
                    },
                    None => {
                        println!("somehow we got an empty continent cell")
                    }
                }
            },
            None => {
                println!("somehow we got an empty continent set")
            },
        }
    }

    let noise_seed: [u8; 4] = seeder.make_seed();
    let noise_seed = u32::from_be_bytes(noise_seed);

    let mut noise = FastNoiseLite::with_seed(noise_seed as i32);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(2));

    DMatrix::from_fn(map_size_x, map_size_y, |row, column| {
        let row_x: f64 = (row as f64) - half_x;
        let row_y: f64 = (column as f64) - half_y;

        let noise_value = noise.get_noise_2d( row_x as f32, row_y as f32);
        let point_x = row_x + (2.0 * noise_value) as f64;
        let point_y = row_y + (2.0 * noise_value) as f64;

        let point = Point {
            x: point_x,
            y: point_y,
        };
        let closest_index = closest_cell(&point, &voronoi_diagram);
        let closest_cell = voronoi_diagram.cell(closest_index);


        if used_cells.contains(&closest_cell.site()) {
            Tile::new(TileType::Grassland, 0.0)
        }
        else {
            Tile::new(TileType::Water, 0.0)
        }
    })
}

#[cfg(test)]
mod tests {
    use voronoice::*;
    use super::closest_cell;

    #[test]
    fn test_closest_cell_site_point() {
        let map_size_x = 100 as f64;
        let map_size_y = 100 as f64;

        let site_1 = Point { x: -50.0, y: 25.0};
        let site_2 = Point { x: -50.0, y: -25.0};
        let site_3 = Point { x: 50.0, y: 25.0};
        let site_4 = Point { x: 50.0, y: -25.0};

        let sites = vec![site_1.clone(), site_2.clone(), site_3.clone(), site_4.clone()];
        let diagram = VoronoiBuilder::default().set_sites(sites)
        .set_bounding_box(BoundingBox::new_centered(map_size_x, map_size_y))
        //.set_lloyd_relaxation_iterations(1)
        .build().expect("Failed to create a voronoi diagram");

        let mut site_1_index = None;
        let mut site_2_index = None;
        let mut site_3_index = None;
        let mut site_4_index = None;

        for cell in diagram.iter_cells() {
            let cell_site = cell.site_position();
            if cell_site.x == site_1.x && cell_site.y == site_1.y {
                site_1_index = Some(cell.site());
            }
            else if cell_site.x == site_2.x && cell_site.y == site_2.y {
                site_2_index = Some(cell.site());
            }
            else if cell_site.x == site_3.x && cell_site.y == site_3.y {
                site_3_index = Some(cell.site());
            }
            else if cell_site.x == site_4.x && cell_site.y == site_4.y {
                site_4_index = Some(cell.site());
            }

       }

        let site_1_index = site_1_index.expect("Expected to find site_1 in the diagram");
        let site_2_index = site_2_index.expect("Expected to find site_1 in the diagram");
        let site_3_index = site_3_index.expect("Expected to find site_1 in the diagram");
        let site_4_index = site_4_index.expect("Expected to find site_1 in the diagram");

        let closest = closest_cell(&site_1, &diagram);
        assert_eq!(closest, site_1_index);
        let closest = closest_cell(&site_2, &diagram);
        assert_eq!(closest, site_2_index);
        let closest = closest_cell(&site_3, &diagram);
        assert_eq!(closest, site_3_index);
        let closest = closest_cell(&site_4, &diagram);
        assert_eq!(closest, site_4_index);

        let near_site_1 = Point { x: site_1.x + 1.0, y: site_1.y + 1.0 };
        let closest = closest_cell(&near_site_1, &diagram);
        assert_eq!(closest, site_1_index);

    }
}