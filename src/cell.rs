use serde::{Serialize, Deserialize};
use crate::forces;
use crate::update;

#[derive(Serialize,Deserialize,Debug,Clone,Copy)]
pub struct Pos {
  pub x: f64,
  pub y: f64
}

#[derive(Serialize,Deserialize,Debug,Clone,Copy)]
pub struct Stress {
  pub a: f64,
  pub b: f64,
  pub c: f64,
  pub d: f64,
}

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Cell {
  pub pos: Pos,
  pub radius: f64,
  pub neighbor_close: Vec<usize>,
  pub neighbor_far: Vec<usize>,
  pub fixed: bool,
  pub stress: Option<f64>,
  pub update: update::UpdateFunc,
  pub force: forces::ForceFunc,
  pub initial_pos: Pos,
  pub strain: Option<Pos>,
  pub tensor_stress: Option<Stress>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ForceLink {
  parties: (usize, usize),
  value: f64,
  relax_length: f64
}

impl Cell {
  pub fn new(x: f64, y: f64, radius: f64) -> Cell  {
    if radius < 0.0 {
      panic!("Radius must be postive");
    }

    Cell {
      pos: Pos { x, y },
      radius,
      neighbor_close: vec![],
      neighbor_far: vec![],
      fixed: false,
      stress: None,
      update: update::UpdateFunc::None,
      force: forces::ForceFunc::None,
      initial_pos: Pos { x, y },
      strain: None,
      tensor_stress: None
    }
  }
}

impl ForceLink {
  pub fn new(p1: usize, p2: usize, val: f64, relax: f64) -> ForceLink {
    ForceLink {
      parties: (p1, p2),
      value: val,
      relax_length: relax
    }
  }
}

impl Pos {
  pub fn norm(&self) -> f64 {
    (num::pow(self.x, 2) + num::pow(self.y, 2)).sqrt()
  }

  pub fn sub(&self, rhs: &Pos) -> Pos {
    Pos {
      x: self.x - rhs.x,
      y: self.y - rhs.y
    }
  }
}
