use voronoice::*;

pub fn distance(point1: &Point, point2: &Point) -> f64 {
    ((point2.x - point1.x).powi(2) + (point2.y - point1.y).powi(2)).sqrt()
}

pub fn shoelace_area<'a>(points: Vec<&Point>) -> f64 {
    let mut area = 0.0;

    let first_point = match points.first() {
        Some(point) => point,
        None => {
            println!("Empty vector??????");
            return area;
        },
    };
    for (index, point) in points.iter().enumerate() {

        let next_point = if index == (points.len() - 1) {
            first_point
        }
        else {
            points[index + 1]
        };

        area += point.x * next_point.y - point.y * next_point.x;
    }

    (area / 2.0).abs()

}
pub fn shoelace_area_of_cell(cell: VoronoiCell) -> f64 {
    shoelace_area(cell.iter_vertices().collect())
}

#[cfg(test)] 
mod tests{
    use super::distance;
    use voronoice::*;
    use rand::prelude::*;
    use rand_seeder::{Seeder, SipHasher};
    use super::shoelace_area;

    #[test]
    fn test_simple_distance() {
        let point1 = Point { x: 0.0, y: 0.0};
        let point2 = Point { x: 0.0, y: 1.0};

        let distance = distance(&point1, &point2);
        assert_eq!(distance, 1.0);
    }

    #[test]
    fn test_shoelace_simple_square() {
        let bottom_left_point = Point { x: 0.0, y: 0.0};
        let top_left_point = Point { x: 0.0, y: 1.0 };
        let top_right_point = Point { x: 1.0, y: 1.0 };
        let bottom_right_point = Point { x: 1.0, y: 0.0 };

        let sides = vec![&bottom_left_point, &top_left_point, &top_right_point, &bottom_right_point];
        assert_eq!(1.0, shoelace_area(sides));

    }
    #[test]
    fn test_shoelace_simple_rect() {
        let bottom_left_point = Point { x: 0.0, y: 0.0};
        let top_left_point = Point { x: 0.0, y: 1.0 };
        let top_right_point = Point { x: 2.0, y: 1.0 };
        let bottom_right_point = Point { x: 2.0, y: 0.0 };

        let sides = vec![&bottom_left_point, &top_left_point, &top_right_point, &bottom_right_point];
        assert_eq!(2.0, shoelace_area(sides));

    }
}