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

pub fn closest_cell(point: &Point, diagram: &Voronoi) -> usize {
    let cell_count = diagram.sites().len();
    let mut closest_cell = diagram.cell(0);
    let mut min_distance = f64::MAX;
    for index in 0..cell_count {
        let cell = diagram.cell(index);
        let cell_distance = distance(point, cell.site_position());
        if cell_distance < min_distance {
            min_distance = cell_distance;
            closest_cell = cell;
        }
    }

    closest_cell.site()
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

pub fn make_continent_cells(diagram: &Voronoi, options: &ContinentOptions, rng: &mut Pcg64) -> Continents {
    let diagram_box = diagram.bounding_box();
    let land_box_width = options.water_padding_percentage * diagram_box.width();
    println!("land_box_width: {land_box_width}");
    let land_box_height = options.water_padding_percentage * diagram_box.height();
    println!("land_box_height: {land_box_height}");
    let land_box = BoundingBox::new(diagram_box.center().clone(), land_box_width, land_box_height);

    
    let mut initial_continent_cells: HashSet<usize> = HashSet::with_capacity(options.num_continents);
    while initial_continent_cells.len() < options.num_continents {
        match diagram.iter_cells().choose(rng) {
            Some(cell) => {
                if land_box.is_inside(cell.site_position()) {
                    initial_continent_cells.insert(cell.site());
                }
                else {
                    println!("Skipping cell in water buffer??");
                }
            },
            None => {
                println!("It's probably time to start adding result types Izzy");
            },
        }
    }

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

    //println!("total area: {}", options.total_area);
    //println!("initial land area: {land_area}");
    //println!("initial land area percentage: {land_area_percentage}");

    while land_area_percentage <= options.land_area_percentage {
        //println!("total area: {}", options.total_area);
        //println!("land area: {land_area}");
        //println!("land area precentage: {land_area_percentage}");
        let continent_cell_set = continents.choose_mut(rng);
        match continent_cell_set {
            Some(cell_set) => {
                let cell_index = cell_set.iter().choose(rng);
                match cell_index {
                    Some(index) => {
                        //println!("index: {index}");
                        let cell = diagram.cell(*index);
                        let mut neighbors: Vec<usize> = cell.iter_neighbors().collect();
                        //println!("neighbors: {neighbors:?}");
                        neighbors.shuffle(rng);
                        //println!("neighbors: {neighbors:?}");
                        //println!("used_cells: {used_cells:?}");

                        for neighbor in neighbors {
                            //println!("neighbor: {neighbor}");
                            let neighbor_cell = diagram.cell(neighbor);
                            if !used_cells.contains(&neighbor) && land_box.is_inside(neighbor_cell.site_position()){
                                //println!("adding neighbor to continents");
                                used_cells.insert(neighbor);
                                //println!("used_cells: {used_cells:?}");
                                cell_set.insert(neighbor);
                                let cell_area =
                                    shoelace_area_of_cell(diagram.cell(neighbor));
                                //println!("neighbor area: {cell_area}");
                                land_area += cell_area;
                                //println!("new land area: {land_area}");
                                land_area_percentage = (land_area / options.total_area as f64) * 100.0;
                                //println!("new land area percentage: {land_area_percentage}");
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

    Continents::new(used_cells, continents, initial_cells)

}

pub fn voronoi_continents(
    map_size_x: usize,
    map_size_y: usize,
    seeder: &mut Seeder,
    cell_count: usize,
) -> DMatrix<Tile> {
    //println!("generating continents");
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
 
    //println!("initial sites: {sites:?}");
    //println!("sites.len: {}", sites.len());
    let voronoi_diagram = VoronoiBuilder::default()
        .set_sites(sites)
        .set_bounding_box(BoundingBox::new_centered(
            map_size_x as f64,
            map_size_y as f64,
        ))
        .set_lloyd_relaxation_iterations(30)
        .build()
        .expect("Failed to build a voronoi diagram");

    //println!("original cell_count: {}", cell_count);
    //cell_count = voronoi_diagram.sites().len();
    //println!("actual cell_count: {}", cell_count);

    let total_area = map_size_x * map_size_y;
    let ideal_land_area_percentage_lower_bound = 29.0;
    //let ideal_land_area_percentage_lower_bound = 2.0;
    //let num_continents: usize = rng.gen_range(4..=9);
    let num_continents: usize = 8;

    let options = ContinentOptions::new(total_area, ideal_land_area_percentage_lower_bound, num_continents, 7.0);
    let continents = make_continent_cells(&voronoi_diagram, &options, &mut rng);

    //println!("generated continents");


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


    DMatrix::from_fn(map_size_x, map_size_y, |row, column| {
        let row_x: f64 = (row as f64) - half_x;
        let row_y: f64 = (column as f64) - half_y;

        let noise_x_value = noise_x.get_noise_2d(row_x as f32, row_y as f32);
        let noise_y_value = noise_y.get_noise_2d(row_x as f32, row_y as f32);
        let point_x = row_x + noise_x_value as f64;
        let point_y = row_y + noise_y_value as f64;
        //println!("row_x: {row_x}, point_x: {point_x}");
        //println!("row_y: {row_y}, point_y: {point_y}");

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
    })
}

#[cfg(test)]
mod tests {
    use super::closest_cell;
    use voronoice::*;

    #[test]
    fn test_closest_cell_site_point() {
        let map_size_x = 100 as f64;
        let map_size_y = 100 as f64;

        let site_1 = Point { x: -50.0, y: 25.0 };
        let site_2 = Point { x: -50.0, y: -25.0 };
        let site_3 = Point { x: 50.0, y: 25.0 };
        let site_4 = Point { x: 50.0, y: -25.0 };

        let sites = vec![
            site_1.clone(),
            site_2.clone(),
            site_3.clone(),
            site_4.clone(),
        ];
        let diagram = VoronoiBuilder::default()
            .set_sites(sites)
            .set_bounding_box(BoundingBox::new_centered(map_size_x, map_size_y))
            //.set_lloyd_relaxation_iterations(1)
            .build()
            .expect("Failed to create a voronoi diagram");

        let mut site_1_index = None;
        let mut site_2_index = None;
        let mut site_3_index = None;
        let mut site_4_index = None;

        for cell in diagram.iter_cells() {
            let cell_site = cell.site_position();
            if cell_site.x == site_1.x && cell_site.y == site_1.y {
                site_1_index = Some(cell.site());
            } else if cell_site.x == site_2.x && cell_site.y == site_2.y {
                site_2_index = Some(cell.site());
            } else if cell_site.x == site_3.x && cell_site.y == site_3.y {
                site_3_index = Some(cell.site());
            } else if cell_site.x == site_4.x && cell_site.y == site_4.y {
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

        let near_site_1 = Point {
            x: site_1.x + 1.0,
            y: site_1.y + 1.0,
        };
        let closest = closest_cell(&near_site_1, &diagram);
        assert_eq!(closest, site_1_index);
    }
}
