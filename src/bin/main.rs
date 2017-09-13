extern crate chrono;
#[macro_use]
extern crate clap;
extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate rustc_internal_docs;
extern crate syslog;
extern crate toml;

use std::env;
use log::LogLevel;
use env_logger::LogBuilder;
use syslog::Facility;
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
        .arg(Arg::with_name("syslog").short("s").long("syslog").help(
            "Log to the system logger instead of stdout (also accepts the USE_SYSLOG env variable)",
        ))
        .get_matches();

    let verbosity = matches.occurrences_of("verbose");
    let use_syslog = matches.is_present("syslog") || env::var("USE_SYSLOG").is_ok();
    init_logger(verbosity, use_syslog);

    let config_file = matches.value_of("config-file").unwrap();
    let toml_contents = helpers::read_file(&config_file).unwrap();
    let config = toml::from_str(&toml_contents).unwrap();

    for line in format!("{:#?}", config).lines() {
        debug!("{}", line);
    }

    config
}

fn init_logger(verbose: u64, use_syslog: bool) {
    let log_level = match verbose {
        0 => LogLevel::Warn,
        1 => LogLevel::Info,
        2 => LogLevel::Debug,
        _ => LogLevel::Trace,
    };

    if use_syslog {
        init_syslog(log_level);
    } else {
        init_env_logger(log_level);
    }
}

fn init_env_logger(level: LogLevel) {
    let mut lb = LogBuilder::new();

    lb.filter(Some("rustc_internal_docs"), level.to_log_level_filter())
        .format(|record| {
            format!(
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

    lb.init().expect("Couldn't initialize env_logger");
}

fn init_syslog(level: LogLevel) {
    syslog::init(
        Facility::LOG_USER,
        level.to_log_level_filter(),
        Some(env!("CARGO_PKG_NAME")),
    ).expect("Couldn't initialize syslog");
}
