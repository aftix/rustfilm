extern crate clap;
extern crate rustfilm;
extern crate ron;
extern crate num;
extern crate rayon;
extern crate num_cpus;
extern crate x264;

use clap::{Arg, App, SubCommand};
use rustfilm::{update, generation, settings, gfx, simulation, cell};
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use rayon::prelude::*;

fn main() {
  let matches = App::new("rustfilm").version("1.0")
                  .author("Wyatt Campbell <wyatt.campbell@utexas.edu>")
                  .about("Simulates physical properties of biofilms")
                  .arg(Arg::with_name("grid")
                    .long("grid")
                    .value_name("FILE")
                    .help("Sets file to input/output grid from")
                    .takes_value(true)
                  )
                  .arg(Arg::with_name("v")
                    .short("v")
                    .multiple(true)
                    .help("Sets level of verbosity")
                  )
                  .subcommand(SubCommand::with_name("generate")
                    .about("Generate a grid")
                    .version("1.0")
                    .author("Wyatt Campbell <wyatt.campbell@utexas.edu>")
                    .arg(Arg::with_name("fixed")
                      .long("fixed")
                      .value_name("FUNC")
                      .help("Choose what fixing funcion to use")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("nrows")
                      .long("nrows")
                      .value_name("USIZE")
                      .help("Choose the number of rows")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("size")
                      .long("size")
                      .value_name("FLOAT")
                      .help("Choose the size of the bacteria")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("spring_k")
                      .long("spring_k")
                      .value_name("FLOAT")
                      .help("Spring constant")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("damping")
                      .long("damping")
                      .value_name("FLOAT")
                      .help("damping constant")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("del_t")
                      .long("del_t")
                      .value_name("FLOAT")
                      .help("Time to run simulation")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("sineamp")
                      .long("sineamp")
                      .value_name("FLOAT")
                      .help("Amplitude of sine wave")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("sineomega")
                      .long("sineomega")
                      .value_name("FLOAT")
                      .help("Omega of sine wave")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("extforce_x")
                      .long("extforce_x")
                      .value_name("FLOAT")
                      .help("External force")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("lj_epsilon")
                      .long("lj_epsilon")
                      .value_name("FLOAT")
                      .help("Lennard-Jones Potential Epsilon")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("lj_sigma")
                      .long("lj_sigma")
                      .value_name("FLOAT")
                      .help("Lennard-Jones Potential Sigma")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("restraint_k")
                      .long("restraint_k")
                      .value_name("FLOAT")
                      .help("Restraint spring constant")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("repl_dist")
                      .long("repl_dist")
                      .help("Repulsion distance")
                      .value_name("FLOAT")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("repl_min")
                      .long("repl_min")
                      .help("Minimum repulsion distance")
                      .value_name("FLOAT")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("repl_epsilon")
                      .long("repl_epsilon")
                      .value_name("FLOAT")
                      .help("Repulsion epsilon")
                      .takes_value(true)
                    )
                  )
                  .subcommand(SubCommand::with_name("simulate")
                    .about("Simulate a grid")
                    .version("1.0")
                    .author("Wyatt Campbell <wyatt.campbell@utexas.edu>")
                    .arg(Arg::with_name("output")
                      .long("output")
                      .value_name("H.264 FILE")
                      .help("Output directory")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("avgstress")
                      .long("avgstress")
                      .value_name("PNG FILE")
                      .help("File to output average stress vs time graph to")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("dist")
                      .long("dist")
                      .value_name("PNG FILE")
                      .help("File to output average displacement vs time graph to")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("xoff")
                      .long("xoff")
                      .value_name("PNG FILE")
                      .help("File to output average x offset vs time graph to")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("yoff")
                      .long("yoff")
                      .value_name("PNG FILE")
                      .help("File to output average y offset vs time graph to")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("stressstrain")
                      .long("stressstrain")
                      .value_name("PNG FILE")
                      .help("File to output average stress vs average strain to")
                      .takes_value(true)
                    )
                  )
                  .get_matches();

  let grid_name = matches.value_of("grid").unwrap_or("grid.dat").to_string();

  if let Some(matches) = matches.subcommand_matches("generate") {
    generate(&grid_name[..], &matches);
  } else if let Some(matches) = matches.subcommand_matches("simulate") {
    simulate(&grid_name[..], &matches);
  } else {
    eprintln!("Need to choose `generate` or `simulate`.");
  }
}

fn generate(grid_name: &str, matches: &clap::ArgMatches) {
  let mut settings = settings::Settings::new();
  if let Some(error) = settings.args(&matches) {
    eprintln!("Error: {}", error);
    return;
  }

  let size = matches.value_of("size").unwrap_or("0.008").to_string();
  let size = size.parse::<f64>();
  if let Err(_e) = size {
    eprintln!("Error parsing size");
    return;
  }
  let size = size.unwrap();
  if size <= 0.0 {
    eprintln!("size must be positive");
    return;
  }

  let fixed = matches.value_of("fixed").unwrap_or("none").to_string().to_lowercase();
  let updatefunc = update::func_enum(&fixed[..]);
  let major_hook = update::enum_major(&updatefunc);
  let minor_hook = update::enum_minor(&updatefunc);

  let grid = generation::generate_offsetgrid(
      &mut settings,
      size,
      major_hook,
      minor_hook
    ).unwrap();

  let settings_ron = ron::to_string(&settings).expect("RONification failed");
  let ron = ron::to_string(&grid).expect("RONification failed");

  let mut file = File::create(grid_name).expect("File creation failed");
  write!(file, "{}\n{}", settings_ron, ron).expect("File writing failed");
}

fn simulate(grid_name: &str, matches: &clap::ArgMatches) {
  let file = File::open(grid_name).expect("Failed to open file");
  let buffered = BufReader::new(file);
  let mut lines: Vec<String> = vec![];

  for line in buffered.lines() {
    lines.push(line.unwrap());
  }

  let settings: settings::Settings = ron::from_str(&lines[0][..]).expect("deRONification failed");
  let grid: Vec<cell::Cell> = ron::from_str(&lines[1][..]).expect("deRONification failed");

  //let mut states = simulation::rk45(&grid, 0.01, 0.001, 0.1, simulation::derivs, &settings);
  //let mut states = simulation::predictor_corrector(&grid, 0.01, simulation::derivs, &settings);
  let mut states = simulation::predictor_corrector_adaptive(&grid, 0.01, 0.001, 0.1, simulation::derivs, &settings);

  let stress: Vec<_> = states.par_iter_mut()
    .map(|tuple| {
      let time = tuple.1;
      let state = &mut tuple.2;
      let avgs = simulation::get_stress(state, time, &settings);
      if -avgs.max_tension > avgs.max_compression {
        (-avgs.max_tension, avgs)
      } else {
        (avgs.max_compression, avgs)
      }
    }).collect();
  let max_stress = stress.iter().max_by(|f1, f2| f1.0.partial_cmp(&f2.0).unwrap()).unwrap().0;
  let max_stress = if max_stress <= 1e-10 { 1.0 } else { max_stress };

  let output = matches.value_of("output").unwrap_or("output.h264").to_string();

  encode(&states, &output, max_stress);

  if let Some(avgstress) = matches.value_of("avgstress") {
    let stress: Vec<_> = states.iter().enumerate().map(|(ind, (_iter, time, _state))| {
        (*time, stress[ind].1.avg_stress)
    }).collect();
    gfx::plot_avgstress(&stress, avgstress);
  }

  let strain: Vec<_> = states.iter_mut().map(|tuple| {
    simulation::get_strain(&mut tuple.2, tuple.1)
  }).collect();

  if let Some(disp) = matches.value_of("dist") {
    let strain: Vec<_> = strain.iter().enumerate().map(|(ind, strain)| {
      (states[ind].1, strain.avgstrain.norm())
    }).collect();
    gfx::plot_dist(&strain, disp);
  }

  if let Some(xoff) = matches.value_of("xoff") {
    let strain: Vec<_> = strain.iter().enumerate().map(|(ind, strain)| {
      (states[ind].1, strain.avgstrain.x)
    }).collect();
    gfx::plot_dist(&strain, xoff);
  }

  if let Some(yoff) = matches.value_of("yoff") {
    let strain: Vec<_> = strain.iter().enumerate().map(|(ind, strain)| {
      (states[ind].1, strain.avgstrain.y)
    }).collect();
    gfx::plot_dist(&strain, yoff);
  }

  if let Some(stressstrain) = matches.value_of("stressstrain") {
    let strain: Vec<_> = strain.iter().enumerate().map(|(ind, strain)| {
      (stress[ind].1.avg_stress, strain.avgstrain.norm())
    }).collect();
    gfx::plot_stressstrain(&strain, stressstrain);
  }
}

fn to_i420(frame: &Vec<u8>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
  let mut y_plane: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE];
  let mut u_plane: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE/4];
  let mut v_plane: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE/4];

  for i in 0..gfx::SIZE*gfx::SIZE {
    let r = frame[i*3] as f64;
    let g = frame[i*3 + 1] as f64;
    let b = frame[i*3 + 2] as f64;

    let y = (0.257 * r) + (0.504 * g) + (0.098 * b) + 16.0;
    let u = -(0.148 * r) - (0.291*g) + (0.439 * b) + 128.0;
    let v = (0.439 * r) - (0.368 * g) - (0.071 * b) + 128.0;

    let y = if y < 0.0 { 0.0 } else if y > 255.0 { 255.0 } else { y };
    let u = if u < 0.0 { 0.0 } else if u > 255.0 { 255.0 } else { u };
    let v = if v < 0.0 { 0.0 } else if v > 255.0 { 255.0 } else { v };

    let y = y as u8;
    let u = u as u8;
    let v = v as u8;
    y_plane[i] = y;

    let row = i % gfx::SIZE;
    let col = i / gfx::SIZE;

    u_plane[(row/2) + (col/2)*gfx::SIZE/2] += u / 4;
    v_plane[(row/2) + (col/2)*gfx::SIZE/2] += v / 4;
  }

  (y_plane, u_plane, v_plane)
}

fn encode(states: &Vec<(i32, f64, Vec<cell::Cell>)>, output: &str, max_stress: f64) {
  let mut par = x264::Param::new();
  par = par.set_dimension(gfx::SIZE, gfx::SIZE);
  par = par.param_parse("repeat_headers", "1").unwrap();
  par = par.param_parse("annexb", "1").unwrap();
  par = par.param_parse("fps", &gfx::FPS.to_string()).unwrap();
  par = par.apply_profile("high").unwrap();

  let mut pic = x264::Picture::from_param(&par).unwrap();

  let mut enc = x264::Encoder::open(&mut par).unwrap();
  let mut output = File::create(output).expect("Unable to open output file");
  let mut timestamp = 0;

  for (_, _, state) in states {
    let frame = to_i420(&gfx::plot_buf(&state, max_stress));
    pic.as_mut_slice(0).unwrap().copy_from_slice(&frame.0);
    pic.as_mut_slice(1).unwrap().copy_from_slice(&frame.1);
    pic.as_mut_slice(2).unwrap().copy_from_slice(&frame.2);

    pic = pic.set_timestamp(timestamp);
    timestamp += 1;
    if let Some((nal, _, _)) = enc.encode(&pic).unwrap() {
      let buf = nal.as_bytes();
      output.write_all(buf).unwrap();
    }
  }

  while enc.delayed_frames() {
    if let Some((nal, _, _)) = enc.encode(None).unwrap() {
      let buf = nal.as_bytes();
      output.write_all(buf).unwrap();
    }
  }
}
