use clap::{self, Arg, App, ArgMatches};

use crate::{util::{log_err}, client::check_special_rpc_values};


pub(crate) const CHUNK_BLOCKS_TASK: &str = "chunk_blocks";
pub(crate) const COUNT_KEY_TXS_TASK: &str = "count_txs";
pub(crate) const MEAN_FILE_SIZE_TASK: &str = "mean_fsize";
pub(crate) const SCRAPE_BLOCKS_TASK: &str = "scrape_blocks";
pub(crate) const COMPARE_BLOCK_LOADS_TASK: &str = "cmp_block_loads";
pub(crate) const BLOCK_SAMPLE_TASK: &str = "block_sample";

pub(crate) static TASK_NAMES: [&str; 6] = [
    CHUNK_BLOCKS_TASK, COUNT_KEY_TXS_TASK, MEAN_FILE_SIZE_TASK,
    SCRAPE_BLOCKS_TASK, COMPARE_BLOCK_LOADS_TASK, BLOCK_SAMPLE_TASK
];

pub(crate) struct CliArguments {
    pub task: String,
    pub minutes: Option<u64>,
    pub rpc: Option<String>,
    pub chunk_size: Option<usize>
}

pub(crate) fn get_cli_args() -> CliArguments {
    let app = App::new("Solana Data Processing Playground")
    .version("0.1.0")
    .author("by: stellz")
    .about("solana data toys: rpc scraping & analysis")
    .arg(Arg::with_name("task")
             .short("t")
             .long("task")
             .takes_value(true)
             .help("Which sub-command to run"))
    .arg(Arg::with_name("minutes")
             .short("m")
             .long("minutes")
             .takes_value(true)
             .required_if("task", SCRAPE_BLOCKS_TASK)
             .help("How long to run the task, in minutes"))
    .arg(Arg::with_name("rpc")
             .short("r")
             .long("rpc")
             .takes_value(true)
             .required_if("task", SCRAPE_BLOCKS_TASK)
             .help("URL of the Solana RPC node to use, or: one of 'dev','test','main'"))
    .arg(Arg::with_name("chunk_mb")
            .short("cm")
            .long("chunk_mb")
            .takes_value(true)
            .required_if("task", CHUNK_BLOCKS_TASK)
            .help("target size (in megabytes) for chunked collections of input data"));

    let matches = app.get_matches();
    let task = matches.value_of("task").unwrap_or("").to_owned();

    let minutes = parse_minutes(&matches);
    let rpc = parse_rpc(&matches);
    let chunk_size = parse_chunk_size(&matches);
    println!("\nparsed chunk size argument:  {}\n", chunk_size.unwrap_or(0));

    CliArguments { task, minutes, rpc, chunk_size }
}

fn parse_minutes(matches: &ArgMatches) -> Option<u64> {
    let raw_minutes = matches.value_of("minutes").unwrap_or("").to_owned();
    if raw_minutes.is_empty() { 
        return None 
    }
    match raw_minutes.parse::<u64>() {
        Ok(m) => Some(m),
        Err(e) => { log_err(&e); None }
    }
}

fn parse_rpc(matches: &ArgMatches) -> Option<String> {
    match matches.value_of("rpc") {
        Some(rpc_arg) => {
            check_special_rpc_values(&rpc_arg)
        },
        None => None,
    }
}

fn parse_chunk_size(matches: &ArgMatches) -> Option<usize> {
    match matches.value_of("chunk_mb") {
        Some(size_str) => {
            match size_str.parse::<usize>() {
                Ok(size) => Some(size),
                Err(e) => { log_err(&e); None }
            }
        },
        None => None
    }
}
