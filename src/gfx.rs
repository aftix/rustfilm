use plotters::prelude::*;

use crate::cell;

// must be even
pub const SIZE: usize = 1024;
pub const FPS: usize = 24;

pub fn plot(grid: &Vec<cell::Cell>, name: &str, max_stress: f64) {
  let drawing_area = BitMapBackend::new(name, (SIZE as u32, SIZE as u32)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let scale = (SIZE as f64) / (1.25 - -0.25);

  grid.iter().for_each(|cell| {
    let rad = cell.radius * scale;
    let pos = ((cell.pos.x + 0.25) * scale, (cell.pos.y + 0.25) * scale);
    let pos = (pos.0 as i32, pos.1 as i32);
    if cell.fixed {
      drawing_area.draw(&Circle::new(pos, rad as i32, Into::<ShapeStyle>::into(&GREEN).filled())).unwrap();
    } else {
      drawing_area.draw(&Circle::new(pos, rad as i32, Into::<ShapeStyle>::into(&BLACK).filled())).unwrap();
    }

    if let Some(stress) = cell.stress {
      let color = if stress > 0.0 {
        let t = 1.0 - stress/max_stress;
        let mix_red = t * 1.0;
        let mix_green = t * 1.0;
        let mix_blue = 1.0;
        RGBColor((mix_red * 255.0) as u8, (mix_green * 255.0) as u8, (mix_blue * 255.0) as u8)
      } else {
        let t = 1.0 - stress/max_stress;
        let mix_red = 1.0;
        let mix_green = t * 1.0;
        let mix_blue = t * 1.0;
        RGBColor((mix_red * 255.0) as u8, (mix_green * 255.0) as u8, (mix_blue * 255.0) as u8)
      };
      drawing_area.draw(&Circle::new(pos, rad as i32 - 1, Into::<ShapeStyle>::into(&color).filled())).unwrap();
    }
  });
}

pub fn plot_buf(grid: &Vec<cell::Cell>, max_stress: f64) -> Vec<u8> {
  let mut rgb: Vec<u8> = vec![];
  for _ in 0..SIZE*SIZE {
    rgb.push(0); //red
    rgb.push(0); //green
    rgb.push(0); //blue
  }

  {
    let drawing_area = BitMapBackend::with_buffer(&mut rgb, (SIZE as u32, SIZE as u32)).into_drawing_area();
    drawing_area.fill(&WHITE).unwrap();

    let scale = (SIZE as f64) / (1.25 - -0.25);

    grid.iter().for_each(|cell| {
      let rad = cell.radius * scale;
      let pos = ((cell.pos.x + 0.25) * scale, (cell.pos.y + 0.25) * scale);
      let pos = (pos.0 as i32, pos.1 as i32);
      if cell.fixed {
        drawing_area.draw(&Circle::new(pos, rad as i32, Into::<ShapeStyle>::into(&GREEN).filled())).unwrap();
      } else {
        drawing_area.draw(&Circle::new(pos, rad as i32, Into::<ShapeStyle>::into(&BLACK).filled())).unwrap();
      }

      if let Some(stress) = cell.stress {
        let color = if stress > 0.0 {
          let t = 1.0 - stress/max_stress;
          let mix_red = t * 1.0;
          let mix_green = t * 1.0;
          let mix_blue = 1.0;
          RGBColor((mix_red * 255.0) as u8, (mix_green * 255.0) as u8, (mix_blue * 255.0) as u8)
        } else {
          let t = 1.0 - stress/max_stress;
          let mix_red = 1.0;
          let mix_green = t * 1.0;
          let mix_blue = t * 1.0;
          RGBColor((mix_red * 255.0) as u8, (mix_green * 255.0) as u8, (mix_blue * 255.0) as u8)
        };
        drawing_area.draw(&Circle::new(pos, rad as i32 - 1, Into::<ShapeStyle>::into(&color).filled())).unwrap();
      }
    });
  }

  rgb
}

pub fn plot_avgstress(avgstress: &Vec<(f64, f64)>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (SIZE as u32, SIZE as u32)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let max_time = avgstress.iter().max_by(|t1, t2| t1.0.partial_cmp(&t2.0).unwrap()).unwrap().0;
  let max_avg = avgstress.iter().max_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;
  let min_avg = avgstress.iter().min_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;

  let mut chart = ChartBuilder::on(&drawing_area)
    .margin(5)
    .set_all_label_area_size(50)
    .caption("Average Stress vs Time", ("sans-serif", 50))
    .build_cartesian_2d(
      0f32..1.25 * max_time as f32,
      if min_avg > 0.0 { 0f32 } else { min_avg as f32 * 1.25 }..1.25 * max_avg as f32
    ).unwrap();

  chart.configure_mesh().x_labels(10).y_labels(10).disable_mesh().draw().unwrap();

  chart.draw_series(PointSeries::of_element(
      avgstress.iter().map(|(t, avg)| (*t as f32, *avg as f32)),
      5,
      ShapeStyle::from(&BLACK).filled(),
      &|coord, size, style| {
        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
      }
  )).unwrap();
}

pub fn plot_dist(avgdist: &Vec<(f64, f64)>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (SIZE as u32, SIZE as u32)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let max_time = avgdist.iter().max_by(|t1, t2| t1.0.partial_cmp(&t2.0).unwrap()).unwrap().0;
  let max_dist = avgdist.iter().max_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;
  let min_dist = avgdist.iter().min_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;

  let mut chart = ChartBuilder::on(&drawing_area)
    .margin(5)
    .set_all_label_area_size(50)
    .caption("Average Displacement vs Time", ("sans-serif", 50))
    .build_cartesian_2d(
      0f32..1.25 * max_time as f32,
      if min_dist > 0.0 { 0f32 } else { min_dist as f32 * 1.25 }..1.25 * max_dist as f32
    ).unwrap();

  chart.configure_mesh().x_labels(10).y_labels(10).disable_mesh().draw().unwrap();

  chart.draw_series(PointSeries::of_element(
      avgdist.iter().map(|(t, avg)| (*t as f32, *avg as f32)),
      5,
      ShapeStyle::from(&BLACK).filled(),
      &|coord, size, style| {
        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
      }
  )).unwrap();
}

pub fn plot_xoff(avgdist: &Vec<(f64, f64)>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (SIZE as u32, SIZE as u32)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let max_time = avgdist.iter().max_by(|t1, t2| t1.0.partial_cmp(&t2.0).unwrap()).unwrap().0;
  let max_dist = avgdist.iter().max_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;
  let min_dist = avgdist.iter().min_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;

  let mut chart = ChartBuilder::on(&drawing_area)
    .margin(5)
    .set_all_label_area_size(50)
    .caption("Average X offset vs Time", ("sans-serif", 50))
    .build_cartesian_2d(
      0f32..1.25 * max_time as f32,
      if min_dist > 0.0 { 0f32 } else { min_dist as f32 * 1.25 }..1.25 * max_dist as f32
    ).unwrap();

  chart.configure_mesh().x_labels(10).y_labels(10).disable_mesh().draw().unwrap();

  chart.draw_series(PointSeries::of_element(
      avgdist.iter().map(|(t, avg)| (*t as f32, *avg as f32)),
      5,
      ShapeStyle::from(&BLACK).filled(),
      &|coord, size, style| {
        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
      }
  )).unwrap();
}

pub fn plot_yoff(avgdist: &Vec<(f64, f64)>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (SIZE as u32, SIZE as u32)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let max_time = avgdist.iter().max_by(|t1, t2| t1.0.partial_cmp(&t2.0).unwrap()).unwrap().0;
  let max_dist = avgdist.iter().max_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;
  let min_dist = avgdist.iter().min_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;

  let mut chart = ChartBuilder::on(&drawing_area)
    .margin(5)
    .set_all_label_area_size(50)
    .caption("Average Y offset vs Time", ("sans-serif", 50))
    .build_cartesian_2d(
      0f32..1.25 * max_time as f32,
      if min_dist > 0.0 { 0f32 } else { min_dist as f32 * 1.25 }..1.25 * max_dist as f32
    ).unwrap();

  chart.configure_mesh().x_labels(10).y_labels(10).disable_mesh().draw().unwrap();

  chart.draw_series(PointSeries::of_element(
      avgdist.iter().map(|(t, avg)| (*t as f32, *avg as f32)),
      5,
      ShapeStyle::from(&BLACK).filled(),
      &|coord, size, style| {
        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
      }
  )).unwrap();
}

// avg stress vs avg strain
pub fn plot_stressstrain(strstr: &Vec<(f64, f64)>, name: &str) {
  let drawing_area = BitMapBackend::new(name, (SIZE as u32, SIZE as u32)).into_drawing_area();
  drawing_area.fill(&WHITE).unwrap();

  let max_stress = strstr.iter().max_by(|t1, t2| t1.0.partial_cmp(&t2.0).unwrap()).unwrap().0;
  let min_stress = strstr.iter().min_by(|t1, t2| t1.0.partial_cmp(&t2.0).unwrap()).unwrap().0;
  let max_strain = strstr.iter().max_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;
  let min_strain = strstr.iter().min_by(|t1, t2| t1.1.partial_cmp(&t2.1).unwrap()).unwrap().1;

  let mut chart = ChartBuilder::on(&drawing_area)
    .margin(5)
    .set_all_label_area_size(50)
    .caption("Average Stress vs Average Strain", ("sans-serif", 50))
    .build_cartesian_2d(
      if min_strain > 0.0 { 0f32 } else { min_strain as f32 * 1.25 }..max_strain as f32,
      if min_stress > 0.0 { 0f32 } else { min_stress as f32 * 1.25 }..1.25 * max_stress as f32
    ).unwrap();

  chart.configure_mesh().x_labels(10).y_labels(10).disable_mesh().draw().unwrap();

  chart.draw_series(PointSeries::of_element(
      strstr.iter().map(|(stress, strain)| (*strain as f32, *stress as f32)),
      5,
      ShapeStyle::from(&BLACK).filled(),
      &|coord, size, style| {
        EmptyElement::at(coord) + Circle::new((0, 0), size, style)
      }
  )).unwrap();
}
