mod helpers;

extern crate env_logger;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::Path;

use clap::{Arg, Command};
use log::LevelFilter;

use surrealdb::engine::local::Mem;
use surrealdb::sql::statements::{BeginStatement, CommitStatement};
use surrealdb::*;

use indicatif::{ProgressBar, ProgressStyle};

//Use jemalloc on *nix
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

use crate::helpers::value_to_table;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    //Command line args
    let matches = Command::new("sqx")
        .version("0.1")
        .author("twin. <hyperViridian@gmail.com>")
        .about("A tool for filtering, selecting, querying, aggregating, or transforming any data with SurrealQL.")
        .allow_hyphen_values(true)
        .arg(
            Arg::new("log-timestamp")
                .short('t')
                .help("Timestamp log, self output")
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
                .help("The path to the input file(s) or folder(s). Can take recursive glob (*) paths.")
                .last(true)
                .num_args(0..)
                .action(clap::ArgAction::Append),
        )
        .arg(
            Arg::new("input-format")
                .short('F')
                .long("if")
                .help("Specify input format for data. Defaults to JSON. Valid options are JSON, TSV, CSV, ARROW, SYSLOG.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("output-format")
                .short('f')
                .long("output-format")
                .help("Specify input format for data. Defaults to TABLED graphical output. Valid options are JSON, TSV, CSV, ARROW, TABLED.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("output-file")
                .short('o')
                .long("of")
                .help("The path to the output file.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("sql-query")
                .short('s')
                .long("sql-query")
                .help("A SurrealQL query to run against the data.")
                .action(clap::ArgAction::Set),
        )
        .get_matches_from(wild::args());

    //Match for verbosity level from args
    let verbosity = match matches.get_count("verbosity") {
        0 => (LevelFilter::Off, "none"),
        1 => (LevelFilter::Error, "error"),
        2 => (LevelFilter::Warn, "warn"),
        3 => (LevelFilter::Debug, "debug"),
        4 => (LevelFilter::Trace, "trace"),
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

    // Spin up the database
    let db = Surreal::new::<Mem>(()).await?;

    info!("In memory datastore initialized.");

    // Iter of all file input sources besides stdin taking into account glob paths
    if let Some(flags) = matches.get_many::<String>("input-path") {
        let true_files: Vec<&Path> = flags
            .filter_map(|str_path| {
                let path = Path::new(str_path);
                if path.is_file() {
                    return Some(path);
                }
                None
            })
            .collect();
        if log_enabled!(log::Level::Trace) {
            true_files.iter().for_each(|path| {
                trace!("Input path: {}", path.to_string_lossy());
            });
        }
        db.use_ns("namespace")
            .use_db("filein")
            .await
            .expect("Failed to change database to filein.");
        let bar = ProgressBar::new(true_files.len().try_into().unwrap()).with_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("##-"),
        );
        for file in true_files {
            let handle = File::open(file)
                .expect(format!("Could not open file: {:#?}", &file.to_path_buf()).as_str());
            let buf_reader = BufReader::new(handle);
            let deserialized: serde_json::Value = serde_json::from_reader(buf_reader).unwrap();
            debug!("Surreal converted json to: {:#?}", deserialized);

            db
                // Start transaction
                .query(BeginStatement)
                // Setup accounts
                .query("INSERT INTO filein $obj;")
                .bind(("obj", deserialized))
                // Finalise
                .query(CommitStatement)
                .await
                .expect("DB Insertion failed");
            bar.inc(1);
        }
        bar.finish_with_message("done");
        if let Some(sql_statement) = matches.get_one::<String>("sql-query") {
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Setup accounts
                .query(sql_statement)
                // Finalise
                .query(CommitStatement)
                .await?;
            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!("{}", value_to_table(responses).unwrap());
        } else {
            info!("No SQL string provided, selecting ALL from filein.");
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Setup accounts
                .query("SELECT * FROM $table;")
                .bind(("table", "filein"))
                // Finalise
                .query(CommitStatement)
                .await?;
            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!("{}", value_to_table(responses).unwrap());
        }
    } else {
        //Stdin

        db.use_ns("namespace")
            .use_db("stdin")
            .await
            .expect("Failed to change database to stdin.");
        let stdin = io::stdin();
        let handle = stdin.lock();
        let buf_reader = BufReader::new(handle);

        let deserialized: serde_json::Value = serde_json::from_reader(buf_reader).unwrap();
        let value = sql::json(&deserialized.to_string()).expect("STDIN JSON was malformed.");
        debug!("Surreal converted json to: {:#?}", value);
        let sql = surrealdb::sql! {
            INSERT INTO stdin $obj
        };

        let results = db.query(sql).bind(("obj", value)).await?;
        debug!("INSERT resulted in: {:#?}", results);

        if let Some(sql_statement) = matches.get_one::<String>("sql-query") {
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Setup accounts
                .query(sql_statement)
                // Finalise
                .query(CommitStatement)
                .await?;
            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!("{}", value_to_table(responses).unwrap());
        } else {
            info!("No SQL string provided, selecting ALL from stdin.");
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Setup accounts
                .query("SELECT * FROM $table;")
                .bind(("table", "stdin"))
                // Finalise
                .query(CommitStatement)
                .await?;
            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!("{}", value_to_table(responses).unwrap());
        }
    };

    Ok(())
}
