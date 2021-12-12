use clap::{self, Arg, App, ArgMatches};

use crate::{util::{log_err, log_err_none, println_each_indent}, networks::expand_rpc_keywords};


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
    pub chunk_size: Option<usize>,
    pub sample_rate: Option<usize>
}

pub(crate) fn get_cli_args() -> CliArguments {
    let app = App::new("Solana Data Processing Playground")
    .version("0.1.0")
    .author("by: stellz")
    .about("solana data toys: rpc scraping & analysis")
    .arg(Arg::with_name("task")
             .long("task")
             .short("t")
             .takes_value(true)
             .required(true)
             .help("Which sub-command to run"))
    .arg(Arg::with_name("minutes")
             .long("minutes")
             .short("m")
             .takes_value(true)
             .required_if("task", SCRAPE_BLOCKS_TASK)
             .help("How long to run the task, in minutes"))
    .arg(Arg::with_name("rpc")
             .long("rpc")
             .short("r")
             .takes_value(true)
             .required_if("task", SCRAPE_BLOCKS_TASK)
             .help("URL of the Solana RPC node to use, or: 1 of 'dev','test','main'"))
    .arg(Arg::with_name("chunk_mb")
            .long("chunk_mb")
            .takes_value(true)
            .required_if("task", CHUNK_BLOCKS_TASK)
            .help("size (megabytes) for chunked collections of input data, default: 2"))
    .arg(Arg::with_name("sample-rate")
            .long("sample-rate")
            .takes_value(true)
            .help("number of source files for each 1 copied to new sample, default: 50"));

    let matches = app.get_matches();

    let task = parse_task(&matches);
    let minutes = parse_minutes(&matches);
    let rpc = parse_rpc(&matches);
    let chunk_size = parse_chunk_size(&matches);
    let sample_rate = parse_sample_rate(&matches);

    CliArguments { task, minutes, rpc, chunk_size, sample_rate }
}

fn parse_task(matches: &ArgMatches) -> String {
    match matches.value_of("task") {
        Some(task_arg) => { 
            if !TASK_NAMES.contains(&task_arg) { 
                eprintln!("\ntask '{}' not recognized!", task_arg);
                println!("available tasks:");
                println_each_indent(&TASK_NAMES, true);  
            }
            task_arg.to_string()
        },
        _ => "".to_string(),
    }
}

fn parse_minutes(matches: &ArgMatches) -> Option<u64> {
    if let Some(minutes_arg) = matches.value_of("minutes") {
        return match minutes_arg.parse::<u64>() {
            Ok(m) => Some(m),
            Err(e) => log_err_none(&e)
        }
    }
    else { None }
}

fn parse_rpc(matches: &ArgMatches) -> Option<String> {
    if let Some(rpc_arg) = matches.value_of("rpc") {
        Some(expand_rpc_keywords(&rpc_arg).to_string())
    } 
    else { None }
}

fn parse_chunk_size(matches: &ArgMatches) -> Option<usize> {
    if let Some(mb_arg) = matches.value_of("chunk_mb") {
        match mb_arg.parse::<usize>() {
            Ok(size) => Some(size),
            Err(e) => { log_err(&e); None }
        }
    } 
    else { None }
}

fn parse_sample_rate(matches: &ArgMatches) -> Option<usize> {
    if let Some(sr_arg) = matches.value_of("sample-rate") {
        match sr_arg.parse::<usize>() {
            Ok(n) => Some(n),
            Err(e) => log_err_none(&e)
        } 
    } 
    else { None }
}
