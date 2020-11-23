use crate::settings;
use crate::cell;
use crate::update;
use std::error::Error;
use std::fmt;
use std::cell::RefCell;

#[derive(Debug)]
pub struct GenerationError {
  pub error: String
}

impl Error for GenerationError {}

impl fmt::Display for GenerationError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(&self.error[..])
  }
}

pub type GridType = RefCell<cell::Cell>;

pub fn generate_offsetgrid(
    params: &mut settings::Settings,
    nrows: usize,
    size: f64,
    major_hook: Option<fn(usize, usize, &mut cell::Cell) -> ()>,
    minor_hook: Option<fn(usize, usize, &mut cell::Cell) -> ()>
  ) -> Result<Vec<GridType>, GenerationError> {
    let mut grid: Vec<GridType> = vec![];

    let needed_space = 2.0 * size * ((2 * nrows - 1) as f64); // Amount of space needed to fit all the cells
    if needed_space > 1.0 {
      return Err(GenerationError { error: "Space needed exceeds limits".to_string() });
    }

    let taken_space = 2.0 * size * (nrows as f64); // Amount of space the cells themselves take up

    let gap_space = (1.0 - taken_space) / (nrows as f64 - 1.0); // Space left over to space out the cells

    // Generate the major rows
    let mut xpos = size;
    let mut ypos = size;
    let iter_space = gap_space + 2.0 * size; // Space between centers

    for i in 0..nrows {
      for j in 0..nrows {
        grid.push(RefCell::new(cell::Cell::new(xpos, ypos, size)));
        if let Some(hook) = major_hook {
          hook(i, j, &mut grid[i * nrows + j].borrow_mut());
        }
        xpos += iter_space;
      }
      xpos = size;
      ypos += iter_space;
    }

    xpos = 2.0 * size + gap_space / 2.0;
    ypos = 2.0 * size + gap_space / 2.0;
    for i in 0..nrows {
      for j in 0..nrows {
        grid.push(RefCell::new(cell::Cell::new(xpos, ypos, size)));
        if let Some(hook) = minor_hook {
          hook(i, j, &mut grid[i * nrows + j].borrow_mut());
        }
        xpos += iter_space;
      }
      xpos = 2.0 * size + gap_space / 2.0;
      ypos += iter_space;
    }

    let big_space = iter_space;
    let small_space = iter_space / 2.0 * 1.4142135623731; //sqrt2

    params.spring_relax_close = small_space;
    params.spring_relax_far = big_space;

    for i in &grid {
      let mut cell = i.borrow_mut();
      cell.neighbor_close = vec![];
      cell.neighbor_far = vec![];

      for (index, j) in grid.iter().enumerate() {
        let other = j.borrow();
        let mydist = cell.pos.sub(&other.pos).norm();
        if mydist == big_space {
          cell.neighbor_far.push(index);
        } else if mydist == small_space {
          cell.neighbor_close.push(index);
        }
      }
    }

    for i in &grid {
      let mut cell = i.borrow_mut();
      if cell.update != update::UpdateFunc::None {
        update::update(&mut cell);
      }
    }

    Ok(grid)
}
