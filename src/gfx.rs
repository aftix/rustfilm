use plotters::prelude::*;

use crate::{cell, generation};

pub fn plot(grid: Vec<generation::GridType>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (1200, 680)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let mut chart = ChartBuilder::on(&drawing_area)
    .set_label_area_size(LabelAreaPosition::Left, 40)
    .set_label_area_size(LabelAreaPosition::Bottom, 40)
    .build_cartesian_2d(-0.25..1.25, -0.25..1.25)
    .unwrap();

  let mut normal_cell: Vec<(f64, f64)> = vec![];
  let mut fixed_cell: Vec<(f64, f64)> = vec![];

  for refcell in &grid {
    let cell = refcell.borrow();
    if cell.fixed {
      fixed_cell.push((cell.pos.x, cell.pos.y));
    } else {
      normal_cell.push((cell.pos.x, cell.pos.y));
    }
  }

  chart.draw_series(
    normal_cell.iter().map(|point| Circle::new(*point, 5, &BLACK))
  ).unwrap();
  chart.draw_series(
    fixed_cell.iter().map(|point| Circle::new(*point, 5, &GREEN))
  ).unwrap();
}
