use serde::{Serialize, Deserialize};
use super::RustfilmError;

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
  pub repl_epsilon: f64,
  pub nrows: usize
}

impl Settings {
  pub fn new() -> Settings {
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
      repl_epsilon: 5.0,
      nrows: 10
    }
  }

  pub fn args(&mut self, matches: &clap::ArgMatches) -> Option<RustfilmError> {
    if let Some(spring_k) = matches.value_of("spring_k") {
      match spring_k.parse::<f64>() {
        Ok(spring_k) => self.spring_k = spring_k,
        Err(_e) => return Some(RustfilmError {error: "spring_k failed to parse".to_string()})
      }
      if self.spring_k <= 0.0 {
        return Some(RustfilmError { error: "spring_k must be positive".to_string() });
      }
    }

    if let Some(damping) = matches.value_of("damping") {
      match damping.parse::<f64>() {
        Ok(damping) => self.damping = damping,
        Err(_e) => return Some(RustfilmError{error: "damping failed to parse".to_string()})
      }
      if self.damping <= 0.0 {
        return Some(RustfilmError{error: "damping must be positive".to_string()});
      }
    }

    if let Some(del_t) = matches.value_of("del_t") {
      match del_t.parse::<f64>() {
        Ok(del_t) => self.del_t = del_t,
        Err(_e) => return Some(RustfilmError{error: "del_t failed to parse".to_string()})
      }
      if self.del_t <= 0.0 {
        return Some(RustfilmError{error: "del_t must be positive".to_string()})
      }
    }

    if let Some(sineamp) = matches.value_of("sineamp") {
      match sineamp.parse::<f64>() {
        Ok(sineamp) => self.sineamp = sineamp,
        Err(_e) => return Some(RustfilmError{error: "sineamp failed to parse".to_string()})
      }
    }

    if let Some(sineomega) = matches.value_of("sineomega") {
      match sineomega.parse::<f64>() {
        Ok(sineomega) => self.sineomega = sineomega,
        Err(_e) => return Some(RustfilmError{error: "sineomega failed to parse".to_string()})
      }
      if self.sineomega <= 0.0 {
        return Some(RustfilmError{error: "sineomega must be positive".to_string()})
      }
    }

    if let Some(extforce_x) = matches.value_of("extforce_x") {
      match extforce_x.parse::<f64>() {
        Ok(extforce_x) => self.extforce_x = extforce_x,
        Err(_e) => return Some(RustfilmError{error: "extforce_x failed to parse".to_string()})
      }
    }

    if let Some(lj_epsilon) = matches.value_of("lj_epsilon") {
      match lj_epsilon.parse::<f64>() {
        Ok(lj_epsilon) => self.lj_epsilon = lj_epsilon,
        Err(_e) => return Some(RustfilmError{error: "lj_epsilon failed to parse".to_string()})
      }
      if self.lj_epsilon <= 0.0 {
        return Some(RustfilmError{error: "lj_epsilon must be positive".to_string()});
      }
    }

    if let Some(lj_sigma) = matches.value_of("lj_sigma") {
      match lj_sigma.parse::<f64>() {
        Ok(lj_sigma) => self.lj_sigma = lj_sigma,
        Err(_e) => return Some(RustfilmError{error: "lj_sigma failed to parse".to_string()})
      }
      if self.lj_sigma <= 0.0 {
        return Some(RustfilmError{error: "lj_sigma must be positive".to_string()});
      }
    }

    if let Some(restraint_k) = matches.value_of("restraint_k") {
      match restraint_k.parse::<f64>() {
        Ok(restraint_k) => self.restraint_k = restraint_k,
        Err(_e) => return Some(RustfilmError{error: "restraint_k failed to parse".to_string()})
      }
      if self.restraint_k < 0.0 {
        return Some(RustfilmError{error: "restraint_k must be nonnegative".to_string()})
      }
    }

    if let Some(repl_dist) = matches.value_of("repl_dist") {
      match repl_dist.parse::<f64>() {
        Ok(repl_dist) => self.repl_dist = repl_dist,
        Err(_e) => return Some(RustfilmError{error: "repl_dist failed to parse".to_string()})
      }
      if self.repl_dist <= 0.0 {
        return Some(RustfilmError{error: "repl_dist must be positive".to_string()});
      }
    }

    if let Some(repl_min) = matches.value_of("repl_min") {
      match repl_min.parse::<f64>() {
        Ok(repl_min) => self.repl_min = repl_min,
        Err(_e) => return Some(RustfilmError{error: "repl_min failed to parse".to_string()})
      }
      if self.repl_min <= 0.0 {
        return Some(RustfilmError{error: "repl_min must be positive".to_string()});
      }
    }

    if let Some(repl_epsilon) = matches.value_of("repl_epsilon") {
      match repl_epsilon.parse::<f64>() {
        Ok(repl_epsilon) => self.repl_epsilon = repl_epsilon,
        Err(_e) => return Some(RustfilmError{error: "repl_epsilon failed to parse".to_string()})
      }
      if self.repl_epsilon < 0.0 {
        return Some(RustfilmError{error: "repl_epsilon must be nonnegative".to_string()});
      }
    }

    if let Some(nrows) = matches.value_of("nrows") {
      match nrows.parse::<usize>() {
        Ok(nrows) => self.nrows = nrows,
        Err(_e) => return Some(RustfilmError{error: "nrows failed to parse".to_string()})
      }
    }

    None
  }
}
