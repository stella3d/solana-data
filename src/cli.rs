use clap::{self, Arg, App, ArgMatches};

use crate::util::log_err;


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
    pub minutes: Option<u64>
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
             .required_if("task", COMPARE_BLOCK_LOADS_TASK)
             .help("How long to run the task, in minutes"));

    let matches = app.get_matches();
    let task = matches.value_of("task").unwrap_or("").to_owned();
    let minutes = parse_minutes(&matches);

    CliArguments { task, minutes }
}

fn parse_minutes(matches: &ArgMatches) -> Option<u64> {
    let raw_minutes = matches.value_of("minutes").unwrap_or("").to_owned();
    match raw_minutes.parse::<u64>() {
        Ok(m) => Some(m),
        Err(e) => { log_err(&e); None }
    }
}