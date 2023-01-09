extern crate env_logger;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use anyhow;

use clap::{Arg, Command};
use log::LevelFilter;

use surrealdb::Datastore;

//Use jemalloc on *nix
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    //Command line args
    let matches = Command::new("sqx")
        .version("0.1")
        .author("twin. <hyperViridian@gmail.com>")
        .about("A tool for filtering, selecting, querying, aggregating, or transforming any data with SurrealQL.")
        .arg(
            Arg::new("log-timestamp")
                .short('t')
                .help("Timestamp log output")
                .action(clap::ArgAction::Count),
        )
        .arg(
            Arg::new("verbosity")
                .short('v')
                .help("Sets the level of verbosity")
                .action(clap::ArgAction::Count),
        )
        .arg(
            Arg::new("input-path")
                .short('i')
                .long("input-path")
                .help("The path to the input file(s) or folder(s). Can take glob (*) paths and be specified multiple times.")
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("stdin")
                .short('s')
                .long("stdin")
                .help("Read input from stdin. This is the default input option if no input paths specified. If specified alongside -i, sqx will aggregate both inputs.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("output-file")
                .short('o')
                .long("of")
                .help("The path to the output file.")
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    //Match for verbosity level from args
    let verbosity = match matches.get_count("verbosity") {
        0 => (LevelFilter::Off, "none"),
        1 => (LevelFilter::Error, "error"),
        2 => (LevelFilter::Warn, "warn"),
        3 => (LevelFilter::Debug, "info"),
        4 => (LevelFilter::Trace, "debug"),
        5 => (LevelFilter::Trace, "trace"),
        _ => (LevelFilter::Info, "info"),
    };

    let mut builder = match matches.get_count("log-timestamp") {
        0 => pretty_env_logger::formatted_builder(),
        1 => pretty_env_logger::formatted_timed_builder(),
        _ => pretty_env_logger::formatted_timed_builder(),
    };

    //Spin up logger
    builder
        .filter_level(verbosity.0)
        .format_timestamp_millis()
        .init();

    info!("Using debug logging level: [{}]", verbosity.1);

    let _ds = Datastore::new("memory").await?;

    info!("In memory datastore initialized.");

    Ok(())
}
