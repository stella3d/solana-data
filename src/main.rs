use test_tasks::test_load_perf_by_size;

use crate::{
    cli::*,
    client::DEVNET_RPC,
    scrape::scrape_loop,
    util::duration_from_hours, 
    files::{BLOCKS_DIR, test_size_average, test_chunk_by_size, test_block_loads, CHUNKED_BLOCKS_DIR}, 
};

pub mod client;
pub mod util;
pub mod analyze;
pub mod files;
pub mod cli;
pub mod test_tasks;
pub mod scrape;

const MEGABYTE: u64 = 1024 * 1024;
const TWO_MEGABYTES: u64 = MEGABYTE * 2;

fn main() {
    let cli_args = get_cli_args();
    match cli_args.task.as_str() {
        SCRAPE_BLOCKS_TASK => {
            // TODO - make network and duration (in minutes) part of cli for this command
            let rpc = DEVNET_RPC;
            scrape_loop(duration_from_hours(12), &rpc);
        },
        CHUNK_BLOCKS_TASK => {
            // TODO - make chunk size part of cli for this command?
            // 2 megabyte chunks tested as by far the fastest to process on my machine
            test_chunk_by_size(TWO_MEGABYTES);
        },
        COUNT_KEY_TXS_TASK => {
            test_block_loads(CHUNKED_BLOCKS_DIR);
        },
        MEAN_FILE_SIZE_TASK => {
            test_size_average(BLOCKS_DIR);
        },
        COMPARE_BLOCK_LOADS_TASK => {
            // TODO - take path as arg
            test_load_perf_by_size("blocks/sized");
        },
        t => { 
            if t.is_empty() { eprintln!("--task / -t argument required to do anything!\n") }
            else { eprintln!("task argument '{}' not recognized!\n", t) }
        }
    }
}
