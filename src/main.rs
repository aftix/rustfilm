extern crate clap;
extern crate rustfilm;
extern crate ron;
extern crate num;
extern crate rayon;

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
                      .value_name("DIR")
                      .help("Output directory")
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

  //let mut states = simulation::rk(&grid, 0.01, simulation::derivs, &settings);
  //let mut states = simulation::rk_adaptive(&grid, 0.01, simulation::derivs, &settings);
  let mut states = simulation::rk45(&grid, 0.01, 0.001, 0.1, simulation::derivs, &settings);

  let max_stress = states.par_iter_mut()
    .map(|tuple| {
      let time = tuple.1;
      let state = &mut tuple.2;
      let avgs = simulation::get_stress(state, time, &settings);
      if -avgs.max_tension > avgs.max_compression { -avgs.max_tension } else { avgs.max_compression }
    }).max_by(|f1, f2| f1.partial_cmp(f2).unwrap()).unwrap();

  let output = matches.value_of("output").unwrap_or("output.ivf").to_string();
  //create_dir(&output).unwrap();

  /*states.par_iter().for_each(|(iter, _time, state)| {
    let name = format!("{}/{:0width$}.png", output, iter, width=5);
    gfx::plot(&state, &name, max_stress);
  });*/

  let frames: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = states.par_iter().map(|(_iter, _time, state)| {
    to_i420(&gfx::plot_buf(&state, max_stress))
  }).collect();
  encode(&frames, &output[..]);
}

extern crate rav1e;
extern crate ivf;
use rav1e::config::SpeedSettings;
use rav1e::*;

// Take interlaced RGB and translate it it 3 planes, Y U V
fn to_i444(frame: &Vec<u8>) -> Vec<u8> {
  let mut planes: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE*3];

  for i in 0..gfx::SIZE*gfx::SIZE {
    let r: f64 = frame[i*3] as f64;
    let g: f64 = frame[i*3+1] as f64;
    let b: f64 = frame[i*3+2] as f64;

    let y = (0.257 * r) + (0.504 * g) + (0.098 * b) + 16.0;
    let u = -(0.148 * r) - (0.291*g) + (0.439 * b) + 128.0;
    let v = (0.439 * r) - (0.368 * g) - (0.071 * b) + 128.0;

    let y = if y < 0.0 { 0.0 } else if y > 255.0 { 255.0 } else { y };
    let u = if u < 0.0 { 0.0 } else if u > 255.0 { 255.0 } else { u };
    let v = if v < 0.0 { 0.0 } else if v > 255.0 { 255.0 } else { v };

    let y = y as u8;
    let u = u as u8;
    let v = v as u8;

    planes[i] = y;
    planes[i + gfx::SIZE*gfx::SIZE] = u;
    planes[i + 2*gfx::SIZE*gfx::SIZE] = v;
  }
  planes
}

fn to_i420(frame: &Vec<u8>) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
  let mut y_plane: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE];
  let mut u_plane: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE/4];
  let mut v_plane: Vec<u8> = vec![0; gfx::SIZE*gfx::SIZE/4];

  let get_yuv = |r: f64, g: f64, b: f64| {
    let y = (0.257 * r) + (0.504 * g) + (0.098 * b) + 16.0;
    let u = -(0.148 * r) - (0.291*g) + (0.439 * b) + 128.0;
    let v = (0.439 * r) - (0.368 * g) - (0.071 * b) + 128.0;

    let y = if y < 0.0 { 0.0 } else if y > 255.0 { 255.0 } else { y };
    let u = if u < 0.0 { 0.0 } else if u > 255.0 { 255.0 } else { u };
    let v = if v < 0.0 { 0.0 } else if v > 255.0 { 255.0 } else { v };

    let y = y as u8;
    let u = u as u8;
    let v = v as u8;

    (y, u, v)
  };

  for row in 0..gfx::SIZE/2 {
    for column in 0..gfx::SIZE/2 {
      let r_1: f64 = frame[(2*row + 2*column*gfx::SIZE) * 3] as f64;
      let g_1: f64 = frame[(2*row + 2*column*gfx::SIZE) * 3 + 1] as f64;
      let b_1: f64 = frame[(2*row + 2*column*gfx::SIZE) * 3 + 2] as f64;

      let r_2: f64 = frame[(2*row + 1 + 2*column*gfx::SIZE) * 3] as f64;
      let g_2: f64 = frame[(2*row + 1 + 2*column*gfx::SIZE) * 3 + 1] as f64;
      let b_2: f64 = frame[(2*row + 1 + 2*column*gfx::SIZE) * 3 + 2] as f64;

      let r_3: f64 = frame[(2*row + (2*column + 1)*gfx::SIZE) * 3] as f64;
      let g_3: f64 = frame[(2*row + (2*column + 1)*gfx::SIZE) * 3 + 1] as f64;
      let b_3: f64 = frame[(2*row + (2*column + 1)*gfx::SIZE) * 3 + 2] as f64;

      let r_4: f64 = frame[(2*row + 1 + (2*column + 1)*gfx::SIZE) * 3] as f64;
      let g_4: f64 = frame[(2*row + 1 + (2*column + 1)*gfx::SIZE) * 3 + 1] as f64;
      let b_4: f64 = frame[(2*row + 1 + (2*column + 1)*gfx::SIZE) * 3 + 2] as f64;

      let yuv_1 = get_yuv(r_1, b_1, g_1);
      let yuv_2 = get_yuv(r_2, b_2, g_2);
      let yuv_3 = get_yuv(r_3, b_3, g_3);
      let yuv_4 = get_yuv(r_4, b_4, g_4);

      let u_sum: u16 = yuv_1.1 as u16 + yuv_2.1 as u16 + yuv_3.1 as u16 + yuv_4.1 as u16;
      let v_sum: u16 = yuv_1.2 as u16 + yuv_2.2 as u16 + yuv_3.2 as u16 + yuv_4.2 as u16;

      y_plane[2*row + 2*column*gfx::SIZE] = yuv_1.0;
      y_plane[2*row + 1 + 2*column*gfx::SIZE] = yuv_2.0;
      y_plane[2*row + (2*column + 1)*gfx::SIZE] = yuv_3.0;
      y_plane[2*row + 1+ (2*column + 1)*gfx::SIZE] = yuv_4.0;

      u_plane[row + column*gfx::SIZE/2] = (u_sum / 4) as u8;
      v_plane[row + column*gfx::SIZE/2] = (v_sum / 4) as u8;
    }
  }

  (y_plane, u_plane, v_plane)
}

// frames in i420
fn encode(frames: &Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>, output: &str) {
  let mut cfg = Config::default();

  cfg.enc.width = gfx::SIZE;
  cfg.enc.height = gfx::SIZE;
  cfg.enc.speed_settings = SpeedSettings::from_preset(9);
  cfg.enc.chroma_sampling = color::ChromaSampling::Cs420;

  let mut ctx: Context<u8> = cfg.new_context().unwrap();

  for frame in frames {
    let mut f = ctx.new_frame();
    f.planes[0].copy_from_raw_u8(&frame.0[..], gfx::SIZE, 1);
    f.planes[1].copy_from_raw_u8(&frame.1[..], gfx::SIZE, 1);
    f.planes[2].copy_from_raw_u8(&frame.2[..], gfx::SIZE, 1);

    match ctx.send_frame(f) {
      Ok(_) => {},
      Err (e) => match e {
        EncoderStatus::EnoughData => panic!("unable to append frame to internal queue"),
        _ => panic!("Unable to send frame")
      }
    }
  }

  ctx.flush();

  let mut file = File::create(output).expect("File creation failed");
  ivf::write_ivf_header(&mut file, gfx::SIZE, gfx::SIZE, gfx::FPS, 1);

  loop {
    match ctx.receive_packet() {
      Ok(pkt) => {
        ivf::write_ivf_frame(&mut file, pkt.input_frameno, &pkt.data[..]);
      },
      Err(e) => match e {
        EncoderStatus::LimitReached => {
          break;
        },
        EncoderStatus::Encoded => {},
        EncoderStatus::NeedMoreData => {
          ctx.send_frame(None).unwrap();
        },
        _ => panic!("Unable to recieve packet")
      }
    };
  }
}
