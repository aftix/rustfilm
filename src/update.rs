use serde::{Serialize, Deserialize};
use crate::{forces, cell};

#[derive(Serialize, Deserialize,Debug,PartialEq,Eq)]
pub enum UpdateFunc {
  None,
  Constrained,
}

pub fn update(cell: &mut cell::Cell) {
  match cell.update {
    UpdateFunc::None => return,
    UpdateFunc::Constrained => return,
  }
}

pub fn func_enum(name: &str) -> UpdateFunc {
  match name {
    "none" => UpdateFunc::None,
    "constrained" => UpdateFunc::Constrained,
    _ => UpdateFunc::None
  }
}

pub fn enum_major(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell) -> ()> {
  match e {
    UpdateFunc::None => None,
    UpdateFunc::Constrained => Some(constrained_major),
  }
}

pub fn enum_minor(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell) -> ()> {
  match e {
    UpdateFunc::None => None,
    UpdateFunc::Constrained => None,
  }
}

pub fn constrained_major(_i: usize, j: usize, c: &mut cell::Cell) {
  if j == 9 {
    c.fixed = true;
  } else if j == 0 {
    c.force = forces::ForceFunc::Constrained;
  }
}
