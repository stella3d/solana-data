use test_tasks::test_basic_subscriptions;

use crate::{
    cli::*, tasks::*,
    client::SolClient, util::log_err,
    scrape::scrape_with_args, 
    input_chunk::chunk_by_size_cli,
    files::{BLOCKS_DIR,  CHUNKED_BLOCKS_DIR, timed_copy_sample},
    test_tasks::{load_perf_by_size, test_size_average, test_block_loads, test_get_block_production},
};

pub mod client; 
pub mod files;
pub mod analyze;
pub mod scrape;
pub mod cli;
pub mod networks;
pub mod tasks;
mod util;
mod test_tasks;
mod input_chunk;

fn main() {
    let cli_args = get_cli_args();

    println!("{}", cli_args.rpc.clone().unwrap_or_default());
    
    // route to various functionality based on the --task arg
    match cli_args.task.as_str() {
        SCRAPE_BLOCKS_TASK =>
            scrape_with_args(&cli_args),
        CHUNK_BLOCKS_TASK =>
            chunk_by_size_cli(&cli_args),
        BLOCK_SAMPLE_TASK => 
            timed_copy_sample(BLOCKS_DIR, cli_args.sample_rate),
        COUNT_KEY_TXS_TASK => 
            test_block_loads(CHUNKED_BLOCKS_DIR),
        MEAN_FILE_SIZE_TASK => 
            test_size_average(BLOCKS_DIR),
        COMPARE_BLOCK_LOADS_TASK =>
            if let Some(s) = cli_args.source { load_perf_by_size(&s) },
        GET_BLOCK_PROD_TASK => {
            match cli_args.rpc {
                Some(rpc) => test_get_block_production(&SolClient::get(&rpc), true),
                None => log_err("CLI parsing should prevent this branch"),
            }
        },
        SUB_BASIC_TASK => {
            let r_url = &cli_args.rpc.unwrap();
            let mut client = SolClient::get(&r_url);
            test_basic_subscriptions(&mut client);
        },
        _ => {}
    }
}
