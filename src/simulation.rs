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
) -> (Vec<f64>, Vec<Vec<generation::GridType>>) {
  let mut path: Vec<Vec<generation::GridType>> = vec![];
  let mut times: Vec<f64> = vec![];
  let state = grid.clone();

  let mut time = 0.0;

  while time < settings.del_t {
    path.push(state.clone().to_vec());
    times.push(time);
    let change = dy(time, &state, &settings);

    for i in 0..state.len() {
      let mut cell = state[i].borrow_mut();
      cell.pos.x += dt * change[i*2];
      cell.pos.y += dt * change[i*2+1];
    }

    time += dt;
  }
  path.push(state);
  times.push(time);

  (times, path)
}

pub struct StressAvg {
  pub max_compression: f64,
  pub max_tension: f64,
  pub avg_stress: f64,
  pub avg_x: f64,
  pub avg_y: f64
}

pub fn get_stress(grid: &Vec<generation::GridType>, t: f64, settings: &settings::Settings) -> StressAvg {
  let mut avgs = StressAvg{
    max_compression: 0.0,
    max_tension: 0.0,
    avg_stress: 0.0,
    avg_x: 0.0,
    avg_y: 0.0
  };

  for refcell in grid {
    let mut cell = refcell.borrow_mut(); cell.tensor_stress = Some(cell::Stress{a: 0.0, b: 0.0, c: 0.0, d: 0.0}); cell.stress = Some(0.0);
  }

  for (i, refcell) in grid.iter().enumerate() {
    let mut cell_a = refcell.borrow_mut();
    for (j, refcell) in grid.iter().enumerate() {
      if i == j {
        continue;
      }
      let cell_b = refcell.borrow();

      let direc = cell_b.pos.sub(&cell_a.pos);
      let dist = direc.norm();

      let mut force = 0.0;

      if cell_a.neighbor_close.contains(&j) {
        force = settings.spring_k * (dist - settings.spring_relax_close);
      } else if cell_a.neighbor_far.contains(&j) {
        force = settings.spring_k * (dist - settings.spring_relax_far);
      }

      if force.abs() < 1e-15 {
        force = 0.0;
      }

      let rhat = cell::Pos{x: direc.x / dist, y: direc.y / dist};
      let force_directed = cell::Pos{x: rhat.x * force, y: rhat.y * force};
      let new_stress = cell::Stress{
        a: force_directed.x * direc.x, b: force_directed.y * direc.x,
        c: force_directed.x * direc.y, d: force_directed.y * direc.y
      };

      if let Some(stress) = cell_a.tensor_stress {
        cell_a.tensor_stress = Some(
          cell::Stress{
            a: stress.a + new_stress.a,
            b: stress.b + new_stress.b,
            c: stress.c + new_stress.c,
            d: stress.d + new_stress.d,
          }
        );
      }
    }

    if cell_a.force != forces::ForceFunc::None {
      let force_func = forces::force_func(&cell_a.force);
      let force = force_func(t, &mut cell_a, &grid, i, settings);
      let force_mag = force.norm();

      if force_mag > 1e-15 {
        let ext_direc = cell::Pos{x: force.x / force_mag, y: force.y / force_mag};
        let ext_direc = cell::Pos{x: ext_direc.x * cell_a.radius, y: ext_direc.y * cell_a.radius};
        if let Some(stress) = cell_a.tensor_stress {
          cell_a.tensor_stress = Some(
            cell::Stress{
              a: stress.a + force.x * ext_direc.x, b: stress.b + force.y * ext_direc.x,
              c: stress.c + force.x * ext_direc.y, d: stress.d + force.y * ext_direc.y
            }
          );
        }
      }
    }

    if let Some(stress) = cell_a.tensor_stress {
      cell_a.tensor_stress = Some(
        cell::Stress{a: stress.a * -0.5, b: stress.b * -0.5, c: stress.c * -0.5, d: stress.d * -0.5}
      );

      let new_a = stress.a * -0.5;
      let new_b = stress.b * -0.5;
      let new = new_a + new_b;

      cell_a.stress = Some(new);
      avgs.avg_x += new_a;
      avgs.avg_y += new_b;
      avgs.avg_stress += new;
    }

    if let Some(stress) = cell_a.stress {
      if stress > avgs.max_compression {
        avgs.max_compression = stress;
      } else if stress < avgs.max_tension {
        avgs.max_tension = stress;
      }
    }
  }

  avgs.avg_stress /= grid.len() as f64;
  avgs.avg_x /= grid.len() as f64;
  avgs.avg_y /= grid.len() as f64;

  avgs
}

pub struct StrainAvg {
  maxdisplace: f64,
  maxxoff: f64,
  maxyoff: f64,
  avgstrain: cell::Pos
}

pub fn get_strain(grid: &Vec<generation::GridType>, _t: f64) -> StrainAvg {
  let mut avgs = StrainAvg{
    maxdisplace: 0.0,
    maxxoff: 0.0,
    maxyoff: 0.0,
    avgstrain: cell::Pos{x: 0.0, y: 0.0}
  };

  for refcell in grid {
    let mut cell = refcell.borrow_mut();
    cell.strain = Some(cell.pos.sub(&cell.initial_pos));

    if let Some(strain) = cell.strain {
      if strain.x > avgs.maxxoff {
        avgs.maxxoff = strain.x;
      }
      if strain.y > avgs.maxyoff {
        avgs.maxyoff = strain.y;
      }
      avgs.avgstrain.x += strain.x;
      avgs.avgstrain.y += strain.y;

      let norm = strain.norm();
      if norm > avgs.maxdisplace {
        avgs.maxdisplace = norm;
      }
    }
  }

  avgs.avgstrain.x /= grid.len() as f64;
  avgs.avgstrain.y /= grid.len() as f64;

  avgs
}
