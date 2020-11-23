extern crate clap;
extern crate rustfilm;
use clap::{Arg, App, SubCommand};
use rustfilm::generation;

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

  let grid_name = matches.value_of("grid").unwrap_or("grid.dat");

  println!("yay");
  if let Some(matches) = matches.subcommand_matches("generate") {
    generate(&grid_name[..], &matches);
  }

  if let Some(matches) = matches.subcommand_matches("simulate") {
    simulate(&grid_name[..], &matches);
  }
}

fn generate(grid_name: &str, matches: &clap::ArgMatches) {
  println!("You chose generate! {}", grid_name);
  generation::generate_offsetgrid();
}

fn simulate(grid_name: &str, matches: &clap::ArgMatches) {
  println!("You chose simulate! {}", grid_name);

}
