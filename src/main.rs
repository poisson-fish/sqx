pub mod converters;
pub mod traits;

use serde_json::Value;
use uuid::Uuid;

extern crate env_logger;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::env::temp_dir;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use clap::{Arg, Command};
use log::LevelFilter;

use surrealdb::engine::local::{Mem, Db};
use surrealdb::sql::statements::{BeginStatement, CommitStatement};
use surrealdb::*;

use indicatif::{ProgressBar, ProgressStyle};

//Use jemalloc on *nix
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

use crate::converters::csv_parse;
use crate::traits::ser_adapter::{FormatOption, FromSerde, ParseSerde};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
                .short('i')
                .long("input-format")
                .help("Specify input format for data. Defaults to JSON. Valid options are JSON, TSV, CSV, ARROW, SYSLOG.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("input-delimiter")
                .short('d')
                .long("input-delimiter")
                .help("Specify delimiter for input data with input-format=CSV. Defaults to ','. Ignored with any other input format.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("output-delimiter")
                .short('D')
                .long("output-delimiter")
                .help("Specify delimiter for output data with *out-format=CSV. Defaults to ','. Ignored with any other output format.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("stdout-format")
                .short('F')
                .long("stdout-format")
                .help("Specify output format for data going to stdout. Defaults to TABLED graphical output. Valid options are JSON, TSV, CSV, ARROW, TABLED.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("fileout-format")
                .short('f')
                .long("fileout-format")
                .help("Specify output format for data going to output file(s). Defaults to TABLED graphical output. Valid options are JSON, TSV, CSV, ARROW, TABLED.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("split")
                .short('S')
                .long("fileout-split")
                .help("Specify a number of lines per file to split on.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("fileout-path")
                .short('o')
                .long("fileout-path")
                .help("The path to the output file.")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("stdout-pretty-print")
                .short('p')
                .long("stdout-pretty-print")
                .help("If specified this flag will enable stdout pretty printing.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("fileout-pretty-print")
                .short('P')
                .long("fileout-pretty-print")
                .help("If specified this flag will enable fileout pretty printing.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("caching")
                .short('c')
                .long("caching")
                .help("Enables query caching. Will cache on ")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file-backed")
                .short('m')
                .long("file-backed")
                .help("Will use an on disk SurrealDB instance instead of in memory.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("db-filepath")
                .short('M')
                .long("db-filepath")
                .help("Specify a filepath to use as the backing SurrealDB database file. For use in conjunction with -m")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("cache-path")
                .short('C')
                .long("cache-path")
                .help("Sets the path of the on disk DB cache. Does nothing if specified with -m.")
                .action(clap::ArgAction::Set),
        )
        .arg(

            Arg::new("query-string")
                .short('s')
                .long("query-string")
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

    let get_format_option = |id: &str, default: FormatOption| match matches.get_one::<String>(id) {
        Some(value) => match value.as_str() {
            "JSON" => FormatOption::JSON,
            "CSV" => FormatOption::CSV,
            "TSV" => FormatOption::TSV,
            "ARROW" => FormatOption::ARROW,
            "TABLED" => FormatOption::TABLED,
            _ => default,
        },
        None => default,
    };
    let stdout_format = get_format_option("stdout-format", FormatOption::TABLED);
    let fileout_format = get_format_option("fileout-format", FormatOption::JSON);
    let input_format = get_format_option("input-format", FormatOption::JSON);
    info!(
        "File Input format is {:#?}, Stdin format is {:#?}, output format {:#?}",
        fileout_format, input_format, stdout_format
    );

    //Spin up logger
    builder
        .filter_level(verbosity.0)
        .format_timestamp_millis()
        .init();

    info!("Using debug logging level: [{}]", verbosity.1);

    let temp_root = match matches.get_one::<String>("cache-path") {
            Some(user_path) => Path::new(user_path),
            None => &Path::join(&temp_dir(),PathBuf::from("/sqx"))
    };
    
    // Spin up the database
    static DB: Surreal<Db> = Surreal::init();
    
    if matches.get_count("file-backed") > 0 {
        let db_path = match matches.get_one("db-filepath") {
            Some(db_userpath) => Path::new(db_userpath),
            None => {
                // generate a uuid filename
                let uuid = Uuid::new_v4();
                &Path::join(temp_root,PathBuf::from(std::format!("{}.db",uuid.to_string())))
            }
        };
        DB.connect::<File>(&"test").await?;
    } else {
        DB.connect::<Mem>(()).await?;
    };    

    info!("In memory datastore initialized.");

    // Iter of all file input sources besides stdin taking into account glob paths
    if let Some(flags) = matches.get_many::<String>("input-path") {
        // We got input files
        db.use_ns("namespace")
            .use_db("filein")
            .await
            .expect("Failed to change database to filein.");

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
            let mut buf_reader = BufReader::new(handle);

            let deserialized: serde_json::Value = buf_reader.parse_serde(&input_format).unwrap();
            debug!("Surreal converted json to: {:#?}", deserialized);

            db
                // Start transaction
                .query(BeginStatement)
                // Insert input data
                .query("INSERT INTO filein $obj;")
                .bind(("obj", deserialized))
                // Finalise
                .query(CommitStatement)
                .await
                .expect("DB Insertion failed");
            bar.inc(1);
        }
        bar.finish_with_message("done");
        if let Some(sql_statement) = matches.get_one::<String>("query-string") {
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Insert statement
                .query(sql_statement)
                // Finalise
                .query(CommitStatement)
                .await?;

            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!(
                "{}",
                responses
                    .to_format_option(&stdout_format)
                    .unwrap_or(String::from("Couldn't generate response string."))
            );
        } else {
            info!("No SQL string provided, selecting ALL from filein.");
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Default Query
                .query("SELECT * FROM $table;")
                .bind(("table", "filein"))
                // Finalise
                .query(CommitStatement)
                .await?;

            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!(
                "{}",
                responses
                    .to_format_option(&stdout_format)
                    .unwrap_or(String::from("Couldn't generate response string."))
            );
        }
    } else {
        //Stdin

        db.use_ns("namespace")
            .use_db("stdin")
            .await
            .expect("Failed to change database to stdin.");

        let stdin = io::stdin();

        let value: Value = match input_format {
            FormatOption::JSON => {
                let handle = stdin.lock();
                let buf_reader = BufReader::new(handle);
                serde_json::from_reader(buf_reader).unwrap()
            }

            FormatOption::CSV => {
                let handle = stdin.lock();
                let mut buf_reader = BufReader::new(handle);
                csv_parse(&mut buf_reader, matches.get_one::<char>("input-delimiter")).unwrap()
            }
            FormatOption::TSV => {
                let handle = stdin.lock();
                let mut buf_reader = BufReader::new(handle);
                csv_parse(&mut buf_reader, Some(&'\t')).unwrap()
            }
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => todo!(),
        };

        debug!("Surreal converted json to: {:#?}", value);

        let sql = surrealdb::sql! {
            INSERT INTO stdin $obj
        };

        let results = db.query(sql).bind(("obj", value)).await?;
        debug!("INSERT resulted in: {:#?}", results);

        if let Some(sql_statement) = matches.get_one::<String>("query-string") {
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Begin statement
                .query(sql_statement)
                // Finalise
                .query(CommitStatement)
                .await?;

            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!(
                "{}",
                responses
                    .to_format_option(&stdout_format)
                    .unwrap_or(String::from("Couldn't generate response string."))
            );
        } else {
            info!("No SQL string provided, selecting ALL from stdin.");
            let mut response = db
                // Start transaction
                .query(BeginStatement)
                // Default query
                .query("SELECT * FROM $table;")
                .bind(("table", "stdin"))
                // Finalise
                .query(CommitStatement)
                .await?;

            let responses: serde_json::Value =
                serde_json::Value::Array(response.take(0).expect("Couldn't deserialize response."));

            println!(
                "{}",
                responses
                    .to_format_option(&stdout_format)
                    .unwrap_or(String::from("Generating string failed."))
            );
        }
    };

    Ok(())
}
