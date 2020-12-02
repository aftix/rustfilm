use serde::{Serialize, Deserialize};
use crate::{forces, cell, settings};

#[derive(Serialize, Deserialize,Debug,PartialEq,Eq,Clone,Copy)]
pub enum UpdateFunc {
  None,
  Constrained,
  Sine,
  Pluck,
}

pub fn update(cell: &mut cell::Cell) {
  match cell.update {
    UpdateFunc::None => return,
    UpdateFunc::Constrained => return,
    UpdateFunc::Sine => return,
    UpdateFunc::Pluck => pluck(cell),
  }
}

pub fn func_enum(name: &str) -> UpdateFunc {
  match name {
    "none" => UpdateFunc::None,
    "constrained" => UpdateFunc::Constrained,
    "sine" => UpdateFunc::Sine,
    "pluck" => UpdateFunc::Pluck,
    _ => UpdateFunc::None
  }
}

pub fn enum_major(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell, &settings::Settings) -> ()> {
  match e {
    UpdateFunc::None => None,
    UpdateFunc::Constrained => Some(constrained_major),
    UpdateFunc::Sine => Some(sine_major),
    UpdateFunc::Pluck => Some(pluck_major),
  }
}

pub fn enum_minor(e: &UpdateFunc) -> Option<fn(usize, usize, &mut cell::Cell, &settings::Settings) -> ()> {
  match e {
    UpdateFunc::None => None,
    UpdateFunc::Constrained => None,
    UpdateFunc::Sine => None,
    UpdateFunc::Pluck => None,
  }
}

pub fn constrained_major(_i: usize, j: usize, c: &mut cell::Cell, s: &settings::Settings) {
  if j == s.nrows - 1 {
    c.fixed = true;
  } else if j == 0 {
    c.force = forces::ForceFunc::Constrained;
  }
}

pub fn sine_major(_i: usize, j: usize, c: &mut cell::Cell, s: &settings::Settings) {
  if j == s.nrows - 1 {
    c.fixed = true;
  } else if j == 0 {
    c.force = forces::ForceFunc::Sine;
  }
}

pub fn pluck_major(i: usize, j: usize, c: &mut cell::Cell, _s: &settings::Settings) {
  if i == 0 && j == 0 {
    c.update = UpdateFunc::Pluck;
    c.fixed = true;
  }
}

fn pluck(c: &mut cell::Cell) {
  c.pos.x -= 0.025;
  c.pos.y -= 0.025;
}
