use serde::{Serialize, Deserialize};
use crate::{forces, cell, settings};

#[derive(Serialize, Deserialize,Debug,PartialEq,Eq,Clone,Copy)]
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

pub fn enum_major(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell, &settings::Settings) -> ()> {
  match e {
    UpdateFunc::None => None,
    UpdateFunc::Constrained => Some(constrained_major),
  }
}

pub fn enum_minor(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell, &settings::Settings) -> ()> {
  match e {
    UpdateFunc::None => None,
    UpdateFunc::Constrained => None,
  }
}

pub fn constrained_major(_i: usize, j: usize, c: &mut cell::Cell, s: &settings::Settings) {
  if j == s.nrows - 1 {
    c.fixed = true;
  } else if j == 0 {
    c.force = forces::ForceFunc::Constrained;
  }
}
