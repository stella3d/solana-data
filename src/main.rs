use crate::{
    cli::*,
    client::DEVNET_RPC,
    scrape::scrape_loop,
    util::{duration_from_hours, log_err}, 
    files::{
        BLOCKS_DIR,  CHUNKED_BLOCKS_DIR, 
        copy_sample, test_size_average, test_chunk_by_size, test_block_loads
    },
    test_tasks::test_load_perf_by_size,
    constants::TWO_MEGABYTES 
};

pub mod client;
pub mod util;
pub mod analyze;
pub mod files;
pub mod cli;
pub mod test_tasks;
pub mod scrape;
pub mod constants;


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
        BLOCK_SAMPLE_TASK => {
            // TODO - take sample rate as arg, maybe src directory
            if let Err(e) = copy_sample(BLOCKS_DIR, 50) {
                log_err(&e)
            };
        },
        t => { 
            if t.is_empty() { eprintln!("--task / -t argument required to do anything!\n") }
            else { eprintln!("task argument '{}' not recognized!\n", t) }
        }
    }
}
