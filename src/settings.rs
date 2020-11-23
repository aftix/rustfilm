use serde::{Serialize, Deserialize};
use clap::{App, Arg};

#[derive(Serialize,Deserialize,Debug)]
pub struct Settings {
  pub spring_k: f64,
  pub spring_relax_close: f64,
  pub spring_relax_far: f64,
  pub damping: f64,
  pub del_t: f64,
  pub sineamp: f64,
  pub sineomega: f64,
  pub extforce_x: f64,
  pub lj_epsilon: f64,
  pub lj_sigma: f64,
  pub restraint_k: f64,
  pub repl_dist: f64,
  pub repl_min: f64,
  pub repl_epsilon: f64
}

pub fn default_settings() -> Settings {
  Settings {
    spring_k: 1.0,
    spring_relax_close: 0.04,
    spring_relax_far: 0.04,
    damping: 1.0,
    del_t: 20.0,
    sineamp: 1.0,
    sineomega: std::f64::consts::PI,
    extforce_x: 0.05,
    lj_epsilon: 0.05,
    lj_sigma: 0.065,
    restraint_k: 10.0,
    repl_dist: 0.012,
    repl_min: 0.01,
    repl_epsilon: 5.0
  }
}
