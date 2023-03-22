pub mod converters;
pub mod traits;
pub mod parsers;
use crate::parsers::netstat::NetstatParser;
use crate::parsers::ps::*;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use serde_json::Value::{self, Array};
use surrealdb::engine::any::Any;
use uuid::Uuid;

extern crate env_logger;
extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use std::io::{self, Read};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use atty::Stream;
use clap::{Arg, Command};
use log::LevelFilter;

use surrealdb::*;

use indicatif::{ProgressBar, ProgressStyle};

//Use jemalloc on *nix
// #[cfg(not(target_env = "msvc"))]
// use tikv_jemallocator::Jemalloc;

use crate::converters::csv_parse;
use crate::traits::ser_adapter::{FormatOption, FromSerde, ParseSerde};

// #[cfg(not(target_env = "msvc"))]
// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

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
                .short('T')
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
        .arg(

            Arg::new("query-table")
                .short('t')
                .long("query-table")
                .help("The table to run the query against, defaults to filein. [filein, stdin]")
                .action(clap::ArgAction::Set),
        )
        .get_matches_from(wild::args());

    //Match for verbosity level from args
    let verbosity = match matches.get_count("verbosity") {
        0 => (LevelFilter::Off, "none"),
        1 => (LevelFilter::Info, "info"),
        2 => (LevelFilter::Error, "error"),
        3 => (LevelFilter::Warn, "warn"),
        4 => (LevelFilter::Debug, "debug"),
        5 => (LevelFilter::Trace, "trace"),
        _ => (LevelFilter::Info, "info"),
    };

    let mut builder = match matches.get_count("log-timestamp") {
        0 => pretty_env_logger::formatted_builder(),
        1 => pretty_env_logger::formatted_timed_builder(),
        _ => pretty_env_logger::formatted_timed_builder(),
    };

    let get_format_option = |id: &str, default: FormatOption| match matches.get_one::<String>(id) {
        Some(value) => match value.to_lowercase().as_str() {
            "json" => FormatOption::JSON,
            "csv" => FormatOption::CSV,
            "tsv" => FormatOption::TSV,
            "arrow" => FormatOption::ARROW,
            "tabled" => FormatOption::TABLED,
            "ps" => FormatOption::PS,
            "netstat" | "ns" => FormatOption::NETSTAT,
            "none" => FormatOption::NONE,
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

    let temp_root: PathBuf = match matches.get_one::<String>("cache-path") {
        Some(user_path) => PathBuf::from(user_path),
        None => {
            let tempdir = std::env::temp_dir();
            info!("Using temp_dir: {}", tempdir.to_str().unwrap());
            tempdir.join("sqx")
        }
    };
    info!("Using cache path: {}", temp_root.to_str().unwrap());
    // Spin up the database
    static DB: Surreal<Any> = Surreal::init();

    if matches.get_flag("file-backed") {
        let db_path = match matches.get_one::<String>("db-filepath") {
            Some(db_userpath) => PathBuf::from(db_userpath),
            None => {
                // generate a uuid filename
                let uuid = Uuid::new_v4();
                temp_root.join(PathBuf::from(uuid.to_string()))
            }
        };
        let str_path = format!("file://{}", db_path.to_str().unwrap());
        DB.connect(str_path.as_str()).await?;
        info!("On disk datastore initialized at: {}", str_path);
    } else {
        DB.connect("memory").await?;
        info!("In memory datastore initialized.");
    };

    DB.use_ns("sqx")
        .use_db("sqx")
        .await
        .expect("Failed to change database table.");

    // Iter of all file input sources besides stdin taking into account glob paths
    if let Some(flags) = matches.get_many::<String>("input-path") {
        // We got input files
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
            let handle = std::fs::File::open(file)
                .expect(format!("Could not open file: {:#?}", &file.to_path_buf()).as_str());
            let mut buf_reader = BufReader::new(handle);

            let deserialized: serde_json::Value = buf_reader.parse_serde(&input_format).unwrap();
            debug!("Surreal converted json to: {:#?}", deserialized);

            DB
                // Insert input data
                .query("INSERT INTO filein $obj;")
                .bind(("obj", deserialized))
                // Finalise
                .await
                .expect("DB Insertion failed");
            bar.inc(1);
        }
        bar.finish_and_clear();
    }
    if atty::isnt(Stream::Stdin) {
        //Stdin
        let value: Option<Value> = match input_format {
            FormatOption::JSON => {
                let stdin = io::stdin();
                let handle = stdin.lock();
                let buf_reader = BufReader::new(handle);
                serde_json::from_reader(buf_reader).ok()
            }

            FormatOption::CSV => {
                let stdin = io::stdin();
                let handle = stdin.lock();
                let mut buf_reader = BufReader::new(handle);
                csv_parse(&mut buf_reader, matches.get_one::<char>("input-delimiter")).ok()
            }
            FormatOption::TSV => {
                let stdin = io::stdin();
                let handle = stdin.lock();
                let mut buf_reader = BufReader::new(handle);
                csv_parse(&mut buf_reader, Some(&'\t')).ok()
            }
            FormatOption::NONE => None,
            FormatOption::ARROW => todo!(),
            FormatOption::TABLED => todo!(),
            FormatOption::PS => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let as_json = PsParser::new(buffer).parse().ok();
                info!("Table converted to json is: \n{:#?}",as_json);
                as_json
            },
            FormatOption::NETSTAT => {
                let mut buffer = String::new();
                io::stdin().read_to_string(&mut buffer)?;
                let as_json = NetstatParser::new(buffer).parse().ok();
                info!("Table converted to json is: \n{:#?}",as_json);
                as_json
            }
        };

        debug!("Converted json to: {:#?}", value);

        if let Some(some_value) = value {
            let results = DB
                .query("INSERT INTO stdin $obj;")
                .bind(("obj", some_value))
                .await?;
            debug!("INSERT resulted in: {:#?}", results);
        }
    }

    let default_query = String::from("SELECT * FROM $table;");
    let default_table = String::from("filein");
    let query_string = matches
        .get_one::<String>("query-string")
        .unwrap_or(&default_query);
    let query_table = matches
        .get_one::<String>("query-table")
        .unwrap_or(&default_table);
    
    let mut response = DB.query(query_string).bind(("table", query_table)).await?;

    let db_out = response.take(0).expect("Couldn't get a response.");
    let responses = Array(db_out);

    println!(
        "{}",
        responses
            .to_format_option(&stdout_format)
            .unwrap_or(String::from("Couldn't generate response string."))
    );
    Ok(())
}
