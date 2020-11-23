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

pub fn func_enum(name: &str) -> UpdateFunc {
  match name {
    "none" => UpdateFunc::None,
    _ => UpdateFunc::None
  }
}

pub fn enum_major(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell) -> ()> {
  match e {
    UpdateFunc::None => None
  }
}

pub fn enum_minor(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell) -> ()> {
  match e {
    UpdateFunc::None => None
  }
}
