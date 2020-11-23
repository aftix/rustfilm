use serde::{Serialize, Deserialize};
use crate::cell;

#[derive(Serialize, Deserialize,Debug,PartialEq,Eq)]
pub enum UpdateFunc {
  None,
}

pub fn update(cell: &mut cell::Cell) {
  if cell.update == UpdateFunc::None {
    return;
  }
}
