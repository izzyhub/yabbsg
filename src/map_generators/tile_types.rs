enum TileType {
    Water,
    Coast,
    Grassland,
    ThickForest,
    LightForest,
    Desert,
    Ice,
}

impl matrix::prelude::Element for TileType {
    fn zero() -> TileType {
        TileType::Water
    }
}