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
