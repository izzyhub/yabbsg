use crate::map_generators::tile_types::TileType;
use bevy::ecs::component::Component;

#[derive(Debug, PartialEq, Clone, Component, Default, Copy)]
pub struct Tile {
    terrain: TileType,
    elevation: f64,
}

impl Tile {
    pub fn new(terrain: TileType, elevation: f64) -> Tile {
        Tile { terrain, elevation }
    }

    pub fn terrain(&self) -> TileType {
        self.terrain
    }

    pub fn elevation(&self) -> f64 {
        self.elevation
    }
}
