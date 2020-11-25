use crate::{forces, cell, settings};
use num;
use rayon::prelude::*;

// Take in grid, return vector with x, y interlaced
pub fn derivs(t: f64, y: &mut Vec<cell::Cell>, settings: &settings::Settings) -> Vec<f64> {
  let mut derivs: Vec<f64> = vec![];

  let lj_a = settings.repl_epsilon * num::pow(settings.repl_min, 12);
  let lj_b = settings.repl_epsilon * num::pow(settings.repl_min, 6);

  for i in 0..y.len() {
    let cell_a = &y[i];
    let mut net_force = y.par_iter().enumerate().map(|(j, cell_b)| {
        let mut net_force = (0.0, 0.0);
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
        return net_force;
    }).reduce(|| (0.0, 0.0), |acc, (force_x, force_y)| {
      (acc.0 + force_x, acc.1 + force_y)
    });

    let mut cell_a = &mut y[i];

    if !cell_a.fixed {
      if cell_a.force != forces::ForceFunc::None {
        let force_func = forces::force_func(&cell_a.force);
        let force = force_func(t, &mut cell_a, i, settings);
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
    grid: &Vec<cell::Cell>,
    dt: f64,
    dy: fn(f64, &mut Vec<cell::Cell>, &settings::Settings) -> Vec<f64>,
    settings: &settings::Settings
) -> Vec<(i32, f64, Vec<cell::Cell>)> {
  let mut path: Vec<(i32, f64, Vec<cell::Cell>)> = vec![];
  let mut state = grid.clone();

  let mut time = 0.0;
  let mut iter = 0;

  while time < settings.del_t {
    path.push((iter, time, state.clone().to_vec()));
    let change = dy(time, &mut state, &settings);

    for i in 0..state.len() {
      let mut cell = &mut state[i];
      cell.pos.x += dt * change[i*2];
      cell.pos.y += dt * change[i*2+1];
    }

    time += dt;
    iter += 1;
  }
  path.push((iter, time, state.clone().to_vec()));
  path
}

pub fn rk(
  grid: &Vec<cell::Cell>,
  dt: f64,
  dy: fn(f64, &mut Vec<cell::Cell>, &settings::Settings) -> Vec<f64>,
  settings: &settings::Settings
) -> Vec<(i32, f64, Vec<cell::Cell>)> {
  let mut path: Vec<(i32, f64, Vec<cell::Cell>)> = vec![];
  let mut state = grid.clone();

  let mut time = 0.0;
  let mut iter = 0;

  while time < settings.del_t {
    path.push((iter, time, state.clone().to_vec()));

    let k1 = dy(time, &mut state, &settings);
    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += dt * k1[i*2] / 2.0;
      cell.pos.y += dt * k1[i*2+1] / 2.0;
    }

    let k2 = dy(time + dt/2.0, &mut state, &settings);
    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + dt * k2[i*2] / 2.0;
      cell.pos.y = path[path.len() - 1].2[i].pos.y + dt * k2[i*2+1] / 2.0;
    }

    let k3 = dy(time + dt/2.0, &mut state, &settings);
    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + dt * k3[i*2];
      cell.pos.y = path[path.len() - 1].2[i].pos.y + dt * k3[i*2+1];
    }

    let k4 = dy(time + dt, &mut state, &settings);

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + (1.0/6.0) * dt * (k1[i*2] + 2.0*k2[i*2] + 2.0*k3[i*2] + k4[i*2]);
      cell.pos.y = path[path.len() - 1].2[i].pos.y + (1.0/6.0) * dt * (k1[i*2+1] + 2.0*k2[i*2+1] + 2.0*k3[i*2+1] + k4[i*2+1]);
    }
    time += dt;
    iter += 1;
  }
  path.push((iter, time, state.clone().to_vec()));
  path
}

pub fn rk45(
  grid: &Vec<cell::Cell>,
  epsilon: f64,
  dy: fn(f64, &mut Vec<cell::Cell>, &settings::Settings) -> Vec<f64>,
  settings: &settings::Settings
) -> Vec<(i32, f64, Vec<cell::Cell>)> {
  let mut path: Vec<(i32, f64, Vec<cell::Cell>)> = vec![];

  let mut time = 0.0;
  let mut iter = 0;
  let mut last_iter = -1;

  let mut state = grid.clone();

  let mut dt = 0.01;

  // coeffecients for RK4(5)
  let a = [0.0, 2.0/9.0, 1.0/3.0, 3.0/4.0, 1.0, 5.0/6.0];
  let b = [[0.0, 0.0, 0.0, 0.0, 0.0],
            [2.0 / 9.0, 0.0, 0.0, 0.0, 0.0],
            [1.0 / 12.0, 1.0 / 4.0, 0.0, 0.0, 0.0],
            [69.0 / 128.0, -243.0/128.0, 135.0/64.0, 0.0, 0.0],
            [-17.0 / 12.0, 27.0 / 4.0, -27.0 / 5.0, 16.0 / 15.0, 0.0],
            [65.0 / 432.0, -5.0 / 16.0, 13.0 / 16.0, 4.0 / 27.0, 5.0/144.0]];
  let ch = [47.0/450.0, 0.0, 12.0/25.0, 32.0/225.0, 1.0/30.0, 6.0/25.0];
  let ct = [-1.0/150.0, 0.0, 3.0/100.0, -16.0/75.0, -1.0/20.0, 6.0/25.0];

  while time < settings.del_t {
    // Only push state if we actually took a step
    if last_iter != iter {
      path.push((iter, time, state.clone().to_vec()));
    }

    let k1 = dy(time + dt*a[0], &mut state, &settings);
    let k1: Vec<f64> = k1.iter().map(|k| dt*k).collect();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += b[1][0] * k1[i*2];
      cell.pos.y += b[1][0] * k1[i*2+1];
    }
    let k2 = dy(time + dt*a[1], &mut state, &settings);
    let k2: Vec<f64> = k2.iter().map(|k| dt*k).collect();
    state = path[path.len() - 1].2.clone();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += b[2][0] * k1[i*2] + b[2][1] * k2[i*2];
      cell.pos.y += b[2][0] * k1[i*2+1] + b[2][1] * k2[i*2+1];
    }
    let k3 = dy(time + dt*a[2], &mut state, &settings);
    let k3: Vec<f64> = k3.iter().map(|k| dt*k).collect();
    state = path[path.len() - 1].2.clone();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += b[3][0] * k1[i*2] + b[3][1] * k2[i*2] + b[3][2] * k3[i*2];
      cell.pos.y += b[3][0] * k1[i*2+1] + b[3][1] * k2[i*2+1] + b[3][2] * k3[i*2 + 1];
    }
    let k4 = dy(time + dt*a[3], &mut state, &settings);
    let k4: Vec<f64> = k4.iter().map(|k| dt*k).collect();
    state = path[path.len() - 1].2.clone();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += b[4][0] * k1[i*2] + b[4][1] * k2[i*2] + b[4][2] * k3[i*2] + b[4][3] * k4[i*2];
      cell.pos.y += b[4][0] * k1[i*2+1] + b[4][1] * k2[i*2+1] + b[4][2] * k3[i*2 + 1] + b[4][3]*k4[i*2+1];
    }
    let k5 = dy(time + dt*a[4], &mut state, &settings);
    let k5: Vec<f64> = k5.iter().map(|k| dt*k).collect();
    state = path[path.len() - 1].2.clone();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += b[5][0] * k1[i*2] + b[5][1] * k2[i*2] + b[5][2] * k3[i*2] + b[5][3] * k4[i*2] + b[5][4] * k5[i*2];
      cell.pos.y += b[5][0] * k1[i*2+1] + b[5][1] * k2[i*2+1] + b[5][2] * k3[i*2 + 1] + b[5][3]*k4[i*2+1] + b[5][4] * k5[i*2];
    }
    let k6 = dy(time + dt*a[5], &mut state, &settings);
    let k6: Vec<f64> = k6.iter().map(|k| dt*k).collect();
    state = path[path.len() - 1].2.clone();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += ch[0] * k1[i*2] + ch[1]*k2[i*2] + ch[2]*k3[i*2] + ch[3]*k4[i*2] + ch[4]*k5[i*2] + ch[5]*k6[i*2];
      cell.pos.y += ch[0] * k1[i*2+1] + ch[1]*k2[i*2+1] + ch[2]*k3[i*2+1] + ch[3]*k4[i*2+1] + ch[4]*k5[i*2+1] + ch[5]*k6[i*2+1];
    }

    let mut error: Vec<f64> = vec![];
    for i in 0..k1.len() {
      error.push((ct[0]*k1[i] + ct[1]*k2[i] + ct[2]*k3[i] + ct[3]*k4[i] + ct[4]*k5[i] + ct[5]*k6[i]).abs());
    }

    let total_error = error.iter().map(|f| f.powi(2)).sum::<f64>().sqrt();
    let dt_new = 0.9 * dt * (epsilon / total_error).powf(0.2);

    last_iter = iter;
    if total_error <= epsilon {
      iter += 1;
      time += dt;
      dt = dt_new;
    } else {
      dt = dt_new;
    }
  }
  path.push((iter, time, state.clone().to_vec()));
  path
}

pub struct Stressavg {
  pub max_compression: f64,
  pub max_tension: f64,
  pub avg_stress: f64,
  pub avg_x: f64,
  pub avg_y: f64
}

pub fn get_stress(grid: &mut Vec<cell::Cell>, t: f64, settings: &settings::Settings) -> Stressavg {
  let mut avgs = Stressavg{
    max_compression: 0.0,
    max_tension: 0.0,
    avg_stress: 0.0,
    avg_x: 0.0,
    avg_y: 0.0
  };

  for cell in grid.iter_mut() {
    cell.tensor_stress = Some(cell::Stress{a: 0.0, b: 0.0, c: 0.0, d: 0.0});
    cell.stress = Some(0.0);
  }

  for i in 0..grid.len() {
    let cell_a = &grid[i];

    let mut new_tensor_stress = cell::Stress{a: 0.0, b: 0.0, c: 0.0, d: 0.0};

    for (j, cell_b) in grid.iter().enumerate() {
      if i == j {
        continue;
      }

      let direc = cell_b.pos.sub(&cell_a.pos);
      let dist = direc.norm();

      let mut force: f64 = 0.0;

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

      new_tensor_stress.a += new_stress.a;
      new_tensor_stress.b += new_stress.b;
      new_tensor_stress.c += new_stress.c;
      new_tensor_stress.d += new_stress.d;
    }

    let mut cell_a = &mut grid[i];
    cell_a.tensor_stress = Some(new_tensor_stress);

    if cell_a.force != forces::ForceFunc::None {
      let force_func = forces::force_func(&cell_a.force);
      let force = force_func(t, &mut cell_a, i, settings);
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

pub struct Strainavg {
  maxdisplace: f64,
  maxxoff: f64,
  maxyoff: f64,
  avgstrain: cell::Pos
}

pub fn get_strain(grid: &mut Vec<cell::Cell>, _t: f64) -> Strainavg {
  let mut avgs = Strainavg{
    maxdisplace: 0.0,
    maxxoff: 0.0,
    maxyoff: 0.0,
    avgstrain: cell::Pos{x: 0.0, y: 0.0}
  };

  for i in 0..grid.len() {
    let mut cell = &mut grid[i];
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
