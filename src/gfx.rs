use plotters::prelude::*;

use crate::{cell, generation};

const SIZE: u32 = 1000;

pub fn plot(grid: &Vec<generation::GridType>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (SIZE, SIZE)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let mut chart = ChartBuilder::on(&drawing_area)
    .set_label_area_size(LabelAreaPosition::Left, 40)
    .set_label_area_size(LabelAreaPosition::Bottom, 40)
    .build_cartesian_2d(-0.25..1.25, -0.25..1.25)
    .unwrap();

  let scale = (SIZE as f64) / (1.25 - -0.25);

  let mut normal_cell: Vec<(f64, f64)> = vec![];
  let mut fixed_cell: Vec<(f64, f64)> = vec![];

  for refcell in grid {
    let cell = refcell.borrow();
    if cell.fixed {
      fixed_cell.push((cell.pos.x, cell.pos.y));
    } else {
      normal_cell.push((cell.pos.x, cell.pos.y));
    }
  }

  chart.draw_series(
    grid.iter().map(
      |refcell| {
        let cell = refcell.borrow();
        let rad = cell.radius * scale;
        if cell.fixed {
          Circle::new((cell.pos.x, cell.pos.y), rad as i32, &GREEN)
        } else {
          Circle::new((cell.pos.x, cell.pos.y), rad as i32, &BLACK)
        }
      }
    )
  ).unwrap();
}
