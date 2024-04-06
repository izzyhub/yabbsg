#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum TileType {
    #[default]
    Water,
    Grassland,
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
