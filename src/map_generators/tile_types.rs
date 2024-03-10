#[derive(Debug, PartialEq, Clone)]
pub enum TileType {
    Grassland,
    Water,
    //Coast,
    //ThickForest,
    //LightForest,
    //Desert,
    //Ice,
}

impl TileType {
    pub fn to_atlas_index(&self) -> u32 {
        match self {
            Self::Grassland => 0,
            Self::Water => 1,
        }
    }
}
