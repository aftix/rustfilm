use crate::settings;
use crate::cell;
use crate::update;
use float_cmp::approx_eq;

use super::RustfilmError;

pub fn generate_offsetgrid(
    settings: &mut settings::Settings,
    size: f64,
    major_hook: Option<fn(usize, usize, &mut cell::Cell, &settings::Settings) -> ()>,
    minor_hook: Option<fn(usize, usize, &mut cell::Cell, &settings::Settings) -> ()>
  ) -> Result<Vec<cell::Cell>, RustfilmError> {
    let mut grid: Vec<cell::Cell> = vec![];

    let needed_space = 2.0 * size * ((2 * settings.nrows - 1) as f64); // Amount of space needed to fit all the cells
    if needed_space > 1.0 {
      return Err(RustfilmError { error: "Space needed exceeds limits".to_string() });
    }

    let taken_space = 2.0 * size * (settings.nrows as f64); // Amount of space the cells themselves take up

    let gap_space = (1.0 - taken_space) / (settings.nrows as f64 - 1.0); // Space left over to space out the cells

    // Generate the major rows
    let mut xpos = size;
    let mut ypos = size;
    let iter_space = gap_space + 2.0 * size; // Space between centers

    for i in 0..settings.nrows {
      for j in 0..settings.nrows {
        grid.push(cell::Cell::new(xpos, ypos, size));
        if let Some(hook) = major_hook {
          hook(i, j, &mut grid[i * settings.nrows + j], settings);
        }
        xpos += iter_space;
      }
      xpos = size;
      ypos += iter_space;
    }

    xpos = 2.0 * size + gap_space / 2.0;
    ypos = 2.0 * size + gap_space / 2.0;
    for i in 0..settings.nrows-1 {
      for j in 0..settings.nrows-1 {
        grid.push(cell::Cell::new(xpos, ypos, size));
        if let Some(hook) = minor_hook {
          hook(i, j, &mut grid[settings.nrows*settings.nrows + i * settings.nrows + j], settings);
        }
        xpos += iter_space;
      }
      xpos = 2.0 * size + gap_space / 2.0;
      ypos += iter_space;
    }

    let big_space = iter_space;
    let small_space = (iter_space / 2.0) * 1.4142135623731; //sqrt2

    settings.spring_relax_close = small_space;
    settings.spring_relax_far = big_space;

    for ind in 0..grid.len() {
      let cell = &grid[ind];
      let mut neighbor_close: Vec<usize> = vec![];
      let mut neighbor_far: Vec<usize> = vec![];

      for (index, other) in grid.iter().enumerate() {
        if ind == index {
          continue;
        }
        let mydist = cell.pos.sub(&other.pos).norm();
        if approx_eq!(f64, mydist, big_space, ulps = 5, epsilon = 0.00005) {
          neighbor_far.push(index);
        } else if approx_eq!(f64, mydist, small_space, ulps = 5, epsilon = 0.00005) {
          neighbor_close.push(index);
        }
      }

      let mut cell = &mut grid[ind];
      cell.neighbor_close = neighbor_close.clone();
      cell.neighbor_far = neighbor_far.clone();
    }

    for mut cell in grid.iter_mut() {
      if cell.update != update::UpdateFunc::None {
        update::update(&mut cell);
      }
    }

    Ok(grid)
}
