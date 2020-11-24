extern crate clap;
extern crate rustfilm;
extern crate ron;

use clap::{Arg, App, SubCommand};
use rustfilm::{update, generation, settings, cell};
use std::fs::File;
use std::io::{Write, BufRead, BufReader};
use std::cell::RefCell;

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
                      .value_name("INT")
                      .help("Choose the number of rows")
                      .takes_value(true)
                    )
                    .arg(Arg::with_name("size")
                      .long("size")
                      .value_name("FLOAT")
                      .help("Choose the size of the bacteria")
                      .takes_value(true)
                    )
                  )
                  .subcommand(SubCommand::with_name("simulate")
                    .about("Simulate a grid")
                    .version("1.0")
                    .author("Wyatt Campbell <wyatt.campbell@utexas.edu>")
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
                  .get_matches();

  let grid_name = matches.value_of("grid").unwrap_or("grid.dat").to_string();
  let mut settings = settings::Settings::new();
  if let Some(error) = settings.args(&matches) {
    eprintln!("Error: {}", error);
    return;
  }


  if let Some(matches) = matches.subcommand_matches("generate") {
    generate(&grid_name[..], &mut settings, &matches);
  } else if let Some(matches) = matches.subcommand_matches("simulate") {
    simulate(&grid_name[..], &matches);
  } else {
    eprintln!("Need to choose `generate` or `simulate`.");
  }
}

fn generate(grid_name: &str, settings: &mut settings::Settings, matches: &clap::ArgMatches) {
  let nrows = matches.value_of("nrows").unwrap_or("10").to_string();
  let nrows = nrows.parse::<usize>();
  if let Err(_e) = nrows {
    eprintln!("Error parsing nrows");
    return;
  }
  let nrows = nrows.unwrap();
  if nrows <= 0 {
    eprintln!("nrows must be positive");
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
      settings,
      nrows,
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
  let grid: Vec<generation::GridType> = ron::from_str(&lines[1][..]).expect("deRONification failed");

  println!("You chose simulate! {}", grid_name);
}
