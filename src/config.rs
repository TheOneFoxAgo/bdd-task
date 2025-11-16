use super::neighbours::Direction;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub first: Vec<(usize, usize, usize)>,
    pub second: Vec<(usize, usize, usize, usize)>,
    pub third: Vec<ThirdCondition>,
    pub forth: Vec<(usize, usize, usize, usize)>,
    pub order: Vec<usize>,
    pub print_solutions: bool,
    pub result_table_width: usize,
}

#[derive(Deserialize)]
pub struct ThirdCondition(
    pub usize,
    pub usize,
    #[serde(with = "DirectionDef")] pub Direction,
    pub usize,
    pub usize,
);

#[derive(Deserialize)]
#[serde(remote = "Direction")]
pub enum DirectionDef {
    #[serde(rename = "left")]
    Left,
    #[serde(rename = "right")]
    Right,
}
