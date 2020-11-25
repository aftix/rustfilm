use plotters::prelude::*;

use crate::cell;

const SIZE: u32 = 1000;

pub fn plot(grid: &Vec<cell::Cell>, name: &str, max_stress: f64) {
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

  for cell in grid {
    if cell.fixed {
      fixed_cell.push((cell.pos.x, cell.pos.y));
    } else {
      normal_cell.push((cell.pos.x, cell.pos.y));
    }
  }

  chart.draw_series(
    grid.iter().map(
      |cell| {
        let rad = (cell.radius * scale) as i32;
        if cell.fixed {
          Circle::new((cell.pos.x, cell.pos.y), rad, &GREEN)
        } else {
          Circle::new((cell.pos.x, cell.pos.y), rad, &BLACK)
        }
      }
    )
  ).unwrap();

  chart.draw_series(
    grid.iter().map(
      |cell| {
        let rad = (cell.radius * scale) as i32 - 1;
        if let Some(stress) = cell.stress {
          let color =
            if stress > 0.0 { BLUE.mix(stress/max_stress).filled() }
            else { RED.mix(-stress/max_stress).filled()};
          Circle::new((cell.pos.x, cell.pos.y), rad, color)
        } else {
          Circle::new((cell.pos.x, cell.pos.y), rad, BLACK.filled())
        }
      }
    )
  ).unwrap();
}
