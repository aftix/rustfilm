use crate::{forces, cell, settings};
use num;
use rayon::prelude::*;

// Take in grid, return vector with x, y interlaced
pub fn derivs(t: f64, y: &mut Vec<cell::Cell>, settings: &settings::Settings) -> Vec<f64> {
  let lj_a = settings.repl_epsilon * num::pow(settings.repl_min, 12);
  let lj_b = settings.repl_epsilon * num::pow(settings.repl_min, 6);

  let grid = y.clone();

  let forces: Vec<(f64, f64)> = y.iter_mut().enumerate().map(|(i, mut cell_a)| {
    let mut net_force = grid.iter().enumerate().map(|(j, cell_b)| {
        let mut net_force = (0.0, 0.0);
        let a_to_b = cell_b.pos.sub(&cell_a.pos);
        let dist = a_to_b.norm();

        let mut force = 0.0;

        if cell_a.neighbor_close.contains(&j) {
          force += settings.spring_k * (dist - settings.spring_relax_close);
        } else if cell_a.neighbor_far.contains(&j) {
          force += settings.spring_k * (dist - settings.spring_relax_far);
        }

        if force.abs() < 1e-7 {
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
    }).fold((0.0, 0.0), |acc, (force_x, force_y)| {
      (acc.0 + force_x, acc.1 + force_y)
    });

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

    net_force
  }).collect();

  let mut derivs: Vec<f64> = vec![];
  for (x, y) in &forces {
    derivs.push(*x); derivs.push(*y);
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

pub fn predictor_corrector(
  grid: &Vec<cell::Cell>,
  dt: f64,
  dy: fn(f64, &mut Vec<cell::Cell>, &settings::Settings) -> Vec<f64>,
  settings: &settings::Settings
) -> Vec<(i32, f64, Vec<cell::Cell>)> {
  let mut path: Vec<(i32, f64, Vec<cell::Cell>)> = vec![];
  let mut state = grid.clone();
  path.push((0, 0.0, state.clone().to_vec()));

  let mut time = 0.0;

  // Get first 3 states with standard Runge-Kutta Method (assumes dt*3 < settings.del_t)
  for i in 0..3 {
    state = path[i].2.clone();
    let mut k1 = dy(time, &mut state, settings);
    k1.iter_mut().for_each(|k| {*k *= dt;});

    state.iter_mut().enumerate().for_each(|(ind, c)| {
      c.pos.x = path[i].2[ind].pos.x + 0.5*k1[ind*2];
      c.pos.y = path[i].2[ind].pos.y + 0.5*k1[ind*2+1];
    });
    let mut k2 = dy(time + dt*0.5, &mut state, &settings);
    k2.iter_mut().for_each(|k| {*k *= dt;});

    state.iter_mut().enumerate().for_each(|(ind, c)| {
      c.pos.x = path[i].2[ind].pos.x + 0.5*k2[ind*2];
      c.pos.y = path[i].2[ind].pos.y + 0.5*k2[ind*2];
    });
    let mut k3 = dy(time + dt*0.5, &mut state, &settings);
    k3.iter_mut().for_each(|k| {*k *= dt;});

    state.iter_mut().enumerate().for_each(|(ind, c)| {
      c.pos.x = path[i].2[ind].pos.x + k3[ind];
      c.pos.y = path[i].2[ind].pos.y + k3[ind];
    });
    let mut k4 = dy(time + dt, &mut state, &settings);
    k4.iter_mut().for_each(|k| {*k*=dt;});

    state.iter_mut().enumerate().for_each(|(ind, c)| {
      c.pos.x = path[i].2[ind].pos.x + (k1[ind*2] + 2.0*k2[ind*2] + 2.0*k3[ind*2] + k4[ind*2]) / 6.0;
      c.pos.y = path[i].2[ind].pos.y + (k1[ind*2+1] + 2.0*k2[ind*2+1] + 2.0*k3[ind*2+1] + k4[ind*2+1]) / 6.0;
    });
    time += dt;
    path.push(((i+1) as i32, time, state.clone().to_vec()));
  }

  let mut iter = 5;


  // Do Adams fourth-order predictor-corrector method
  while time < settings.del_t {
    time += dt;

    let mut state1 = path[path.len() - 1].2.clone(); // w3
    let mut state2 = path[path.len() - 2].2.clone(); // w2
    let mut state3 = path[path.len() - 3].2.clone(); // w1
    let mut state4 = path[path.len() - 4].2.clone(); // w0

    let f1 = dy(path[path.len() - 1].1, &mut state1, &settings);
    let f2 = dy(path[path.len() - 2].1, &mut state2, &settings);
    let f3 = dy(path[path.len() - 3].1, &mut state3, &settings);
    let f4 = dy(path[path.len() - 4].1, &mut state4, &settings);

    // Predictor
    state.iter_mut().enumerate().for_each(|(ind, c)| {
      c.pos.x = state1[ind].pos.x + dt * (55.0 * f1[ind*2] - 59.0 * f2[ind*2] + 37.0*f3[ind*2] - 9.0*f4[ind*2]) / 24.0;
      c.pos.y = state1[ind].pos.y + dt * (55.0 * f1[ind*2+1] - 59.0 * f2[ind*2+1] + 37.0*f3[ind*2+1] - 9.0*f4[ind*2+1]) / 24.0;
    });
    // Corrector
    let f = dy(time, &mut state, &settings);
    state.iter_mut().enumerate().for_each(|(ind, c)| {
      c.pos.x = state1[ind].pos.x + dt * (9.0 * f[ind*2] + 19.0*f1[ind*2] - 5.0*f2[ind*2] + f3[ind*2])/24.0;
      c.pos.y = state1[ind].pos.y + dt * (9.0 * f[ind*2+1] + 19.0*f1[ind*2+1] - 5.0*f2[ind*2+1] + f3[ind*2+1])/24.0;
    });
    path.push((iter, time, state.clone().to_vec()));
    iter += 1;
  }

  path
}

pub fn rk_adaptive(
  grid: &Vec<cell::Cell>,
  tol: f64,
  dy: fn(f64, &mut Vec<cell::Cell>, &settings::Settings) -> Vec<f64>,
  settings: &settings::Settings
) -> Vec<(i32, f64, Vec<cell::Cell>)> {
  let mut path: Vec<(i32, f64, Vec<cell::Cell>)> = vec![];
  let mut state = grid.clone();

  let mut time = 0.0;
  let mut iter = 0;
  let mut iter_last = -1;

  let mut dt = 0.01;

  while time < settings.del_t {
    if iter_last != iter {
      path.push((iter, time, state.clone().to_vec()));
    }
    iter_last = iter;

    let estimate = |dt: f64, state: &mut Vec<cell::Cell>| {
      let k1 = dy(time, state, &settings);
      for (i, cell) in state.iter_mut().enumerate() {
        cell.pos.x += dt * k1[i*2] / 2.0;
        cell.pos.y += dt * k1[i*2+1] / 2.0;
      }

      let k2 = dy(time + dt/2.0, state, &settings);
      for (i, cell) in state.iter_mut().enumerate() {
        cell.pos.x = path[path.len() - 1].2[i].pos.x + dt * k2[i*2] / 2.0;
        cell.pos.y = path[path.len() - 1].2[i].pos.y + dt * k2[i*2+1] / 2.0;
      }

      let k3 = dy(time + dt/2.0, state, &settings);
      for (i, cell) in state.iter_mut().enumerate() {
        cell.pos.x = path[path.len() - 1].2[i].pos.x + dt * k3[i*2];
        cell.pos.y = path[path.len() - 1].2[i].pos.y + dt * k3[i*2+1];
      }

      let k4 = dy(time + dt, state, &settings);

      for (i, cell) in state.iter_mut().enumerate() {
        cell.pos.x = path[path.len() - 1].2[i].pos.x + (1.0/6.0) * dt * (k1[i*2] + 2.0*k2[i*2] + 2.0*k3[i*2] + k4[i*2]);
        cell.pos.y = path[path.len() - 1].2[i].pos.y + (1.0/6.0) * dt * (k1[i*2+1] + 2.0*k2[i*2+1] + 2.0*k3[i*2+1] + k4[i*2+1]);
      }
    };

    let mut full_step = path[path.len() - 1].2.clone();
    estimate(dt, &mut full_step);

    let mut half_step = path[path.len() - 1].2.clone();
    estimate(dt * 0.5, &mut half_step);

    let error = half_step.iter().zip(full_step.iter()).map(|(half, full)| {
      let err_x = half.pos.x - full.pos.x;
      let err_y = half.pos.y - full.pos.y;
      err_x.powi(2) + err_y.powi(2)
    }).sum::<f64>().sqrt();

    if error > tol { // half step size if error is bad
      dt *= 0.5;
      continue;
    }

    // Check to see if you should double step size
    let mut double_step = path[path.len() - 1].2.clone();
    estimate(dt * 2.0, &mut double_step);

    let error = half_step.iter().zip(double_step.iter()).map(|(half, full)| {
      let err_x = half.pos.x - full.pos.x;
      let err_y = half.pos.y - full.pos.y;
      err_x.powi(2) + err_y.powi(2)
    }).sum::<f64>().sqrt();

    if error <= tol { // If double_step is good, use the double step and continue to
      dt *= 2.0;
      state = double_step;
    } else {
      state = full_step;
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
  dt_min: f64,
  dt_max: f64,
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
  let a = [0.0, 1.0/4.0, 3.0/8.0, 12.0/13.0, 1.0, 1.0/2.0];
  let b = [[0.0, 0.0, 0.0, 0.0, 0.0],
            [1.0 / 4.0, 0.0, 0.0, 0.0, 0.0],
            [3.0 / 32.0, 9.0/32.0, 0.0, 0.0, 0.0],
            [1932.0/2197.0, -7200.0/2197.0, 7296.0/2197.0, 0.0, 0.0],
            [439.0/216.0, -8.0, 3680.0/513.0, -845.0/4104.0, 0.0],
            [-8.0/27.0, 2.0, -3544.0/2565.0, 1859.0/4104.0, -11.0/40.0]];
  let c = [16.0 / 135.0, 0.0, 6656.0/12825.0, 28561.0/56430.0, -9.0/50.0, 2.0/55.0];
  let d = [25.0/216.0, 0.0, 1408.0/2565.0, 2197.0/4104.0, -1.0/5.0, 0.0];

  while time < settings.del_t {
    // Only push state if we actually took a step
    if last_iter != iter {
      path.push((iter, time, state.clone().to_vec()));
    }
    last_iter = iter;

    let k1 = dy(time + dt*a[0], &mut state, &settings);
    let k1: Vec<f64> = k1.iter().map(|k| dt*k).collect();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + b[1][0] * k1[i*2];
      cell.pos.y = path[path.len() - 1].2[i].pos.y + b[1][0] * k1[i*2+1];
    }
    let k2 = dy(time + dt*a[1], &mut state, &settings);
    let k2: Vec<f64> = k2.iter().map(|k| dt*k).collect();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + b[2][0] * k1[i*2] + b[2][1] * k2[i*2];
      cell.pos.y = path[path.len() - 1].2[i].pos.y + b[2][0] * k1[i*2+1] + b[2][1] * k2[i*2+1];
    }
    let k3 = dy(time + dt*a[2], &mut state, &settings);
    let k3: Vec<f64> = k3.iter().map(|k| dt*k).collect();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + b[3][0] * k1[i*2] + b[3][1] * k2[i*2] + b[3][2] * k3[i*2];
      cell.pos.y = path[path.len() - 1].2[i].pos.y + b[3][0] * k1[i*2+1] + b[3][1] * k2[i*2+1] + b[3][2] * k3[i*2 + 1];
    }
    let k4 = dy(time + dt*a[3], &mut state, &settings);
    let k4: Vec<f64> = k4.iter().map(|k| dt*k).collect();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + b[4][0] * k1[i*2] + b[4][1] * k2[i*2] + b[4][2] * k3[i*2] + b[4][3] * k4[i*2];
      cell.pos.y = path[path.len() - 1].2[i].pos.y + b[4][0] * k1[i*2+1] + b[4][1] * k2[i*2+1] + b[4][2] * k3[i*2 + 1] + b[4][3]*k4[i*2+1];
    }
    let k5 = dy(time + dt*a[4], &mut state, &settings);
    let k5: Vec<f64> = k5.iter().map(|k| dt*k).collect();

    for (i, cell) in state.iter_mut().enumerate() {
      cell.pos.x += path[path.len() - 1].2[i].pos.x + b[5][0] * k1[i*2] + b[5][1] * k2[i*2] + b[5][2] * k3[i*2] + b[5][3] * k4[i*2] + b[5][4] * k5[i*2];
      cell.pos.y += path[path.len() - 1].2[i].pos.y + b[5][0] * k1[i*2+1] + b[5][1] * k2[i*2+1] + b[5][2] * k3[i*2 + 1] + b[5][3]*k4[i*2+1] + b[5][4] * k5[i*2];
    }
    let k6 = dy(time + dt*a[5], &mut state, &settings);
    let k6: Vec<f64> = k6.iter().map(|k| dt*k).collect();

    let fifth_order: Vec<f64> = k1.iter().enumerate().map(|(i, k1)| {
      if i % 2 == 0 {
        path[path.len() - 1].2[i/2].pos.x + c[0] * k1 + c[1] * k2[i] + c[2] * k3[i] + c[3] * k4[i] + c[4] * k5[i] + c[5]*k6[i]
      } else {
        path[path.len() - 1].2[i/2].pos.y + c[0] * k1 + c[1] * k2[i] + c[2] * k3[i] + c[3] * k4[i] + c[4] * k5[i] + c[5]*k6[i]
      }
    }).collect();
    let fourth_order: Vec<f64> = k1.iter().enumerate().map(|(i, k1)| {
      if i % 2 == 0 {
        path[path.len() - 1].2[i/2].pos.x + d[0] * k1 + d[1] * k2[i] + d[2] * k3[i] + d[3] * k4[i] + d[4] * k5[i] + d[5]*k6[i]
      } else {
        path[path.len() - 1].2[i/2].pos.y + d[0] * k1 + d[1] * k2[i] + d[2] * k3[i] + d[3] * k4[i] + d[4] * k5[i] + d[5]*k6[i]
      }
    }).collect();

    state.iter_mut().enumerate().for_each(|(i, cell)| {
      cell.pos.x = path[path.len() - 1].2[i].pos.x + d[0] * k1[i*2] + d[1] * k2[i*2] + d[2] * k3[i*2] + d[3] * k4[i*2] + d[4] * k5[i*2] + d[5]*k6[i*2];
      cell.pos.y = path[path.len() - 1].2[i].pos.y + d[0] * k1[i*2+1] + d[1] * k2[i*2+1] + d[2] * k3[i*2+1] + d[3] * k4[i*2+1] + d[4] * k5[i*2+1] + d[5]*k6[i*2+1];
    });

    let error = fifth_order.iter().zip(fourth_order.iter()).map(|(five, four)| {
      (five - four).powi(2)
    }).sum::<f64>().sqrt();

    if error <= epsilon || dt <= dt_min {
      iter += 1;
      time += dt;
    }
    dt *= 0.9 * (epsilon / error).powf(0.2);
    if dt > dt_max {
      dt = dt_max;
    } else if dt < dt_min {
      dt = dt_min;
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
  let grid_old = grid.clone();

  for cell in grid.iter_mut() {
    cell.tensor_stress = Some(cell::Stress{a: 0.0, b: 0.0, c: 0.0, d: 0.0});
    cell.stress = Some(0.0);
  }

  let mut avgs = grid.par_iter_mut().enumerate().map(|(i, mut cell_a)| {
    let mut avgs = Stressavg{
      max_compression: 0.0,
      max_tension: 0.0,
      avg_stress: 0.0,
      avg_x: 0.0,
      avg_y: 0.0
    };

    let new_tensor_stress = grid_old.iter().enumerate().map(|(j, cell_b)| {
      if i == j {
        return cell::Stress{a: 0.0, b: 0.0, c: 0.0, d: 0.0};
      }

      let direc = cell_b.pos.sub(&cell_a.pos);
      let dist = direc.norm();

      let mut force: f64 = 0.0;

      if cell_a.neighbor_close.contains(&j) {
        force = settings.spring_k * (dist - settings.spring_relax_close);
      } else if cell_a.neighbor_far.contains(&j) {
        force = settings.spring_k * (dist - settings.spring_relax_far);
      }

      if force.abs() < 1e-7 {
        force = 0.0;
      }

      let rhat = cell::Pos{x: direc.x / dist, y: direc.y / dist};
      let force_directed = cell::Pos{x: rhat.x * force, y: rhat.y * force};
      let new_stress = cell::Stress{
        a: force_directed.x * direc.x, b: force_directed.y * direc.x,
        c: force_directed.x * direc.y, d: force_directed.y * direc.y
      };

      new_stress
    }).fold(cell::Stress{a: 0.0, b: 0.0, c: 0.0, d: 0.0}, |acc, s| {
      cell::Stress{
        a: acc.a + s.a, b: acc.b + s.b,
        c: acc.c + s.c, d: acc.d + s.d
      }
    });

    cell_a.tensor_stress = Some(new_tensor_stress);

    if cell_a.force != forces::ForceFunc::None {
      let force_func = forces::force_func(&cell_a.force);
      let force = force_func(t, &mut cell_a, i, settings);
      let force_mag = force.norm();

      if force_mag > 1e-7 {
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
      avgs.avg_x = new_a;
      avgs.avg_y = new_b;
      avgs.avg_stress = new;
    }

    if let Some(stress) = cell_a.stress {
      if stress > avgs.max_compression {
        avgs.max_compression = stress;
      } else if stress < avgs.max_tension {
        avgs.max_tension = stress;
      }
    }

    avgs
  }).reduce(|| Stressavg {max_compression: 0.0, max_tension: 0.0, avg_stress: 0.0, avg_x: 0.0, avg_y: 0.0}, |acc, a| {
    let mut ret = Stressavg {
      max_compression: a.max_compression,
      max_tension: a.max_tension,
      avg_stress: acc.avg_stress + a.avg_stress,
      avg_x: acc.avg_x + a.avg_x,
      avg_y: acc.avg_y + a.avg_y
    };

    if acc.max_compression > ret.max_compression {
      ret.max_compression = acc.max_compression;
    }
    if acc.max_tension < ret.max_tension { //tension is negative
      ret.max_tension = acc.max_tension;
    }

    ret
  });

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
  let mut avgs = grid.par_iter_mut().map(|mut cell| {
    let mut avgs = Strainavg{
      maxdisplace: 0.0,
      maxxoff: 0.0,
      maxyoff: 0.0,
      avgstrain: cell::Pos{x: 0.0, y: 0.0}
    };

    cell.strain = Some(cell.pos.sub(&cell.initial_pos));

    if let Some(strain) = cell.strain {
      avgs.maxxoff = strain.x;
      avgs.maxyoff = strain.y;
      avgs.avgstrain.x = strain.x;
      avgs.avgstrain.y = strain.y;
      let norm = strain.norm();
      avgs.maxdisplace = norm;
    }

    avgs
  }).reduce(|| Strainavg{maxdisplace: 0.0, maxxoff: 0.0, maxyoff: 0.0, avgstrain: cell::Pos{x: 0.0, y: 0.0}}, |acc, s| {
    let mut avgs = Strainavg {
      maxdisplace: s.maxdisplace,
      maxxoff: s.maxxoff,
      maxyoff: s.maxyoff,
      avgstrain: cell::Pos{
        x: acc.avgstrain.x + s.avgstrain.x,
        y: acc.avgstrain.y + s.avgstrain.y
      }
    };

    if acc.maxxoff > avgs.maxxoff {
      avgs.maxxoff = acc.maxxoff;
    }
    if acc.maxyoff > avgs.maxyoff {
      avgs.maxyoff = acc.maxyoff;
    }
    if acc.maxdisplace > avgs.maxdisplace {
      avgs.maxdisplace = acc.maxdisplace;
    }

    avgs
  });

  avgs.avgstrain.x /= grid.len() as f64;
  avgs.avgstrain.y /= grid.len() as f64;
  avgs
}
