pub mod cell;
pub mod forces;
pub mod update;
pub mod settings;
pub mod generation;
pub mod gfx;

use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct RustfilmError {
  pub error: String
}

impl Error for RustfilmError {}

impl fmt::Display for RustfilmError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(&self.error[..])
  }
}
