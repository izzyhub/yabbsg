use voronoice::*;
use rand_seeder::Seeder;

pub fn distance(point1: &Point, point2: &Point) -> f64 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2)).sqrt()
}

pub fn shoelace_area(points: Vec<&Point>) -> f64 {
    let mut area = 0.0;

    let first_point = match points.first() {
        Some(point) => point,
        None => {
            println!("Empty vector??????");
            return area;
        }
    };
    for (index, point) in points.iter().enumerate() {
        let next_point = if index == (points.len() - 1) {
            first_point
        } else {
            points[index + 1]
        };

        area += point.x * next_point.y - point.y * next_point.x;
    }

    (area / 2.0).abs()
}
pub fn shoelace_area_of_cell(cell: VoronoiCell) -> f64 {
    shoelace_area(cell.iter_vertices().collect())
}

pub fn create_new_seed32(seeder: &mut Seeder) -> u32 {
    let seed: [u8; 4] = seeder.make_seed();
    u32::from_be_bytes(seed)
}

pub fn create_new_seed64(seeder: &mut Seeder) -> u64 {
    let seed: [u8; 8] = seeder.make_seed();
    u64::from_be_bytes(seed)
}

pub fn land_box(diagram_box: &BoundingBox, water_padding_percentage: f64) -> BoundingBox {
    let land_box_width = water_padding_percentage * diagram_box.width();
    let land_box_height = water_padding_percentage * diagram_box.height();

    BoundingBox::new(diagram_box.center().clone(), land_box_width, land_box_height)
}

pub fn is_cell_in_box(cell: &VoronoiCell, bounding_box: &BoundingBox) -> bool {
    if !bounding_box.is_inside(cell.site_position()) {
        return false;
    }
    for vertex in cell.iter_vertices() {
        if !bounding_box.is_inside(vertex) {
            return false;
        }
    }
    return true;
}

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


#[cfg(test)]
mod tests {
    use super::distance;
    use super::shoelace_area;
    use super::closest_cell;
    use voronoice::*;

    #[test]
    fn test_simple_distance() {
        let point1 = Point { x: 0.0, y: 0.0 };
        let point2 = Point { x: 0.0, y: 1.0 };

        let distance = distance(&point1, &point2);
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_shoelace_simple_square() {
        let bottom_left_point = Point { x: 0.0, y: 0.0 };
        let top_left_point = Point { x: 0.0, y: 1.0 };
        let top_right_point = Point { x: 1.0, y: 1.0 };
        let bottom_right_point = Point { x: 1.0, y: 0.0 };

        let sides = vec![
            &bottom_left_point,
            &top_left_point,
            &top_right_point,
            &bottom_right_point,
        ];
        assert_eq!(1.0, shoelace_area(sides));
    }
    #[test]
    fn test_shoelace_simple_rect() {
        let bottom_left_point = Point { x: 0.0, y: 0.0 };
        let top_left_point = Point { x: 0.0, y: 1.0 };
        let top_right_point = Point { x: 2.0, y: 1.0 };
        let bottom_right_point = Point { x: 2.0, y: 0.0 };

        let sides = vec![
            &bottom_left_point,
            &top_left_point,
            &top_right_point,
            &bottom_right_point,
        ];
        assert_eq!(2.0, shoelace_area(sides));
    }


    #[test]
    fn test_closest_cell_site_point() {
        let map_size_x: f64 = 100.0;
        let map_size_y: f64 = 100.0;

        let site_0 = Point { x: -50.0, y: 25.0 };
        let site_1 = Point { x: -50.0, y: -25.0 };
        let site_2 = Point { x: 50.0, y: 25.0 };
        let site_3 = Point { x: 50.0, y: -25.0 };

        let sites = vec![
            site_0.clone(),
            site_1.clone(),
            site_2.clone(),
            site_3.clone(),
        ];
        let diagram = VoronoiBuilder::default()
            .set_sites(sites)
            .set_bounding_box(BoundingBox::new_centered(map_size_x, map_size_y))
            //.set_lloyd_relaxation_iterations(0)
            .build()
            .expect("Failed to create a voronoi diagram");

        let mut site_0_index = None;
        let mut site_1_index = None;
        let mut site_2_index = None;
        let mut site_3_index = None;

        for cell in diagram.iter_cells() {
            let cell_site = cell.site_position();
            if cell_site.x == site_0.x && cell_site.y == site_0.y {
                site_0_index = Some(cell.site());
            } else if cell_site.x == site_1.x && cell_site.y == site_1.y {
                site_1_index = Some(cell.site());
            } else if cell_site.x == site_2.x && cell_site.y == site_2.y {
                site_2_index = Some(cell.site());
            } else if cell_site.x == site_3.x && cell_site.y == site_3.y {
                site_3_index = Some(cell.site());
            }
        }

        let site_0_index = site_0_index.expect("Expected to find site_0 in the diagram");
        let site_1_index = site_1_index.expect("Expected to find site_1 in the diagram");
        let site_2_index = site_2_index.expect("Expected to find site_2 in the diagram");
        let site_3_index = site_3_index.expect("Expected to find site_3 in the diagram");

        let closest = closest_cell(&site_0, &diagram);
        assert_eq!(closest, site_0_index);
        let closest = closest_cell(&site_1, &diagram);
        assert_eq!(closest, site_1_index);
        let closest = closest_cell(&site_2, &diagram);
        assert_eq!(closest, site_2_index);
        let closest = closest_cell(&site_3, &diagram);
        assert_eq!(closest, site_3_index);

        let near_site_0 = Point {
            x: site_0.x + 1.0,
            y: site_0.y + 1.0,
        };
        let closest = closest_cell(&near_site_0, &diagram);
        assert_eq!(closest, site_0_index);
    }

}
