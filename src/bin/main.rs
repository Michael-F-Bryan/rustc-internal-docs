extern crate chrono;
#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rustc_internal_docs;
extern crate toml;

use std::env;
use std::io::Write;
use log::Level;
use env_logger::Builder;
use clap::Arg;
use chrono::Local;
use rustc_internal_docs::Config;
use rustc_internal_docs::helpers;



fn main() {
    let args = parse_args();
    backtrace!(rustc_internal_docs::run(args));
}

/// Parse command line arguments, initialize the logger and load the config
/// file.
fn parse_args() -> Config {
    // let the default config be either in the home dir or fall back to current dir
    let default_config_location = Config::default_config_file().display().to_string();

    let matches = app_from_crate!()
        .arg(
            Arg::with_name("config-file")
                .short("c")
                .long("config")
                .help("The config file for rustc-internal-docs")
                .default_value(&default_config_location),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the verbosity level (repeat for more verbosity)"),
        )
        .get_matches();

    let verbosity = matches.occurrences_of("verbose");
    init_logger(verbosity);

    let config_file = matches.value_of("config-file").unwrap();
    let toml_contents = helpers::read_file(&config_file).unwrap();
    let config = toml::from_str(&toml_contents).unwrap();

    for line in format!("{:#?}", config).lines() {
        debug!("{}", line);
    }

    config
}

fn init_logger(verbose: u64) {
    let log_level = match verbose {
        0 => Level::Warn,
        1 => Level::Info,
        2 => Level::Debug,
        _ => Level::Trace,
    };

    let mut lb = Builder::new();

    lb.filter(Some("rustc_internal_docs"), log_level.to_level_filter())
        .format(|buf, record| {
            write!(buf,
                "{} [{:5}] - {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        });

    // make sure we still accept the RUST_LOG env variable
    if let Ok(filter) = env::var("RUST_LOG") {
        lb.parse(&filter);
    }

    lb.init();
}

