use serde::{Serialize, Deserialize};
use crate::{cell, generation, settings};

#[derive(Serialize, Deserialize,Debug,PartialEq,Eq, Clone)]
pub enum ForceFunc {
  None,
  Constrained,
}

pub fn force_func(e: &ForceFunc) ->
  fn(
      f64,
      &mut cell::Cell,
      &Vec<generation::GridType>,
      usize,
      &settings::Settings
  ) -> cell::Pos
{
  match e {
    ForceFunc::None => force_none,
    ForceFunc::Constrained => force_constrained,
  }
}

pub fn force_none(
    _t: f64,
    _c: &mut cell::Cell,
    _g: &Vec<generation::GridType>,
    _i: usize,
    _s: &settings::Settings
) -> cell::Pos {
  cell::Pos{x: 0.0, y: 0.0}
}

pub fn force_constrained(
  t: f64,
  c: &mut cell::Cell,
  g: &Vec<generation::GridType>,
  i: usize,
  s: &settings::Settings
) -> cell::Pos {
  let constraint = linear_restraint(t, c, g, i, s);
  let mut force_x = 0.0;

  c.fixed = true;

  if (t as i32) % 10 == 0 {
    c.fixed = false;
    force_x = s.extforce_x;
  }

  cell::Pos{
    x: constraint.x + force_x,
    y: constraint.y
  }
}

pub fn linear_restraint(
  _t: f64,
  c: &mut cell::Cell,
  _g: &Vec<generation::GridType>,
  _i: usize,
  s: &settings::Settings
) -> cell::Pos {
  let mut y_force = 0.0;
  if c.pos.y > 1.0 {
    y_force = s.restraint_k * (1.0 - c.pos.y);
  } else if c.pos.y < 0.0 {
    y_force = s.restraint_k * -c.pos.y;
  }

  cell::Pos {
    x: 0.0,
    y: y_force
  }
}

pub fn lj_restraint(
  _t: f64,
  c: &mut cell::Cell,
  _g: &Vec<generation::GridType>,
  _i: usize,
  s: &settings::Settings
) -> cell::Pos {
  let mut y_force = 0.0;

  if c.pos.y > 1.0 {
    y_force = -4.0 * s.lj_epsilon * (12.0 * s.lj_sigma.powi(12) * (c.pos.y - 1.0).powi(-13) - 6.0 * s.lj_sigma.powi(6) * (c.pos.y - 1.0).powi(-7));
  } else if c.pos.y < 0.0 {
    y_force = 4.0 * s.lj_epsilon * (12.0 * s.lj_sigma.powi(12) * c.pos.y.powi(-13) - 6.0 * s.lj_sigma.powi(6) * c.pos.y.powi(-7));
  }

  cell::Pos {
    x: 0.0,
    y: y_force
  }
}
