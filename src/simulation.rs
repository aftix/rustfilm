use crate::{forces, cell, settings, generation};
use num;

// Take in grid, return vector with x, y interlaced
pub fn derivs(t: f64, y: &Vec<generation::GridType>, settings: &settings::Settings) -> Vec<f64> {
  let mut derivs: Vec<f64> = vec![];

  let lj_a = settings.repl_epsilon * num::pow(settings.repl_min, 12);
  let lj_b = settings.repl_epsilon * num::pow(settings.repl_min, 6);

  for (i, cell_a) in y.iter().enumerate() {
    let mut cell_a = cell_a.borrow_mut();
    let mut net_force = (0.0, 0.0);
    for (j, cell_b) in y.iter().enumerate() {
      if j == i {
        continue;
      }
      let cell_b = cell_b.borrow();
      let a_to_b = cell_b.pos.sub(&cell_a.pos);
      let dist = a_to_b.norm();

      let mut force = 0.0;

      if cell_a.neighbor_close.contains(&j) {
        force += settings.spring_k * (dist - settings.spring_relax_close);
      } else if cell_a.neighbor_far.contains(&j) {
        force += settings.spring_k * (dist - settings.spring_relax_far);
      }

      if force.abs() < 1e-15 {
        force = 0.0;
      }

      if dist < settings.repl_dist {
        force += 12.0 * lj_a * dist.powi(-13) - lj_b * dist.powi(-7);
      }

      let unit_dist = cell::Pos{
        x: a_to_b.x / dist,
        y: a_to_b.y / dist
      };
      let check_nan = unit_dist.x.is_nan() || unit_dist.y.is_nan();
      let check_inf = unit_dist.x.is_infinite() || unit_dist.y.is_infinite();
      let check_nan = check_nan || force.is_nan();
      let check_inf = check_inf || force.is_infinite();
      if !check_nan && !check_inf {
        net_force.0 += force * unit_dist.x;
        net_force.1 += force * unit_dist.y;
      }
    }

    if !cell_a.fixed {
      if cell_a.force != forces::ForceFunc::None {
        let force_func = forces::force_func(&cell_a.force);
        let force = force_func(t, &mut cell_a, &y, i, settings);
        net_force.0 += force.x;
        net_force.1 += force.y;
      }
    } else {
      net_force = (0.0, 0.0);
    }

    net_force.0 /= settings.damping;
    net_force.1 /= settings.damping;

    derivs.push(net_force.0);
    derivs.push(net_force.1);
  }

  derivs
}

pub fn euler(
    grid: &Vec<generation::GridType>,
    dt: f64,
    dy: fn(f64, &Vec<generation::GridType>, &settings::Settings) -> Vec<f64>,
    settings: &settings::Settings
) -> Vec<Vec<generation::GridType>> {
  let mut path: Vec<Vec<generation::GridType>> = vec![];
  let state = grid.clone();

  let mut time = 0.0;

  while time < settings.del_t {
    path.push(state.clone().to_vec());
    let change = dy(time, &state, &settings);

    for i in 0..state.len() {
      let mut cell = state[i].borrow_mut();
      cell.pos.x += dt * change[i*2];
      cell.pos.y += dt * change[i*2+1];
    }

    time += dt;
  }
  path.push(state);

  path
}
