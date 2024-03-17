use crate::map_generators::tile_types::TileType;

#[derive(Debug, PartialEq, Clone)]
pub struct Tile {
    terrain: TileType,
    elevation: f64,
}

impl Tile {
    pub fn new(terrain: TileType, elevation: f64) -> Tile {
        Tile {
            terrain,
            elevation,
        }
    }

    pub fn terrain(&self) -> TileType {
        self.terrain
    }
}