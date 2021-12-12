use crate::{
    cli::*,
    scrape::scrape_with_args,
    util::MEGABYTE, 
    files::{
        BLOCKS_DIR,  CHUNKED_BLOCKS_DIR, 
        timed_copy_sample, test_size_average, test_block_loads, 
    },
    test_tasks::{load_perf_by_size, test_chunk_by_size},
};

pub mod client; 
pub mod files;
pub mod analyze;
pub mod scrape;
pub mod util;
pub mod cli;
pub mod test_tasks;
pub mod networks;

fn main() {
    let cli_args = get_cli_args();
    
    // route to various functionality based on the --task arg
    match cli_args.task.as_str() {
        SCRAPE_BLOCKS_TASK =>
            scrape_with_args(&cli_args),
        CHUNK_BLOCKS_TASK => {
            // TODO - move handling of default values to a step between main() and cli arg parsing
            let mut size = MEGABYTE * 2 as u64;
            if let Some(s) = cli_args.chunk_size {
                size = MEGABYTE * s as u64;
            }
            test_chunk_by_size(size);
        },
        BLOCK_SAMPLE_TASK => {
            let mut rate: usize = 50;       
            if let Some(sr) = cli_args.sample_rate { rate = sr; }
            timed_copy_sample(BLOCKS_DIR, rate);
        },
        COUNT_KEY_TXS_TASK => 
            test_block_loads(CHUNKED_BLOCKS_DIR),
        MEAN_FILE_SIZE_TASK => 
            test_size_average(BLOCKS_DIR),
        COMPARE_BLOCK_LOADS_TASK => 
            load_perf_by_size("blocks/sized"),
        _ => {}
    }
}
