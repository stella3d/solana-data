use crate::{
    cli::*,
    scrape::scrape_with_args, 
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
            if let Some(size) = cli_args.chunk_size {
                test_chunk_by_size(util::MEGABYTE * size as u64)
            } else {
                // 2mb chunks tested by far the best on my dev machine 
                test_chunk_by_size(util::MEGABYTE * 2 as u64)
            }
        },
        COUNT_KEY_TXS_TASK => 
            test_block_loads(CHUNKED_BLOCKS_DIR),
        MEAN_FILE_SIZE_TASK => 
            test_size_average(BLOCKS_DIR),
        COMPARE_BLOCK_LOADS_TASK => 
            load_perf_by_size("blocks/sized"),
        // TODO - take sample rate as arg, maybe src directory
        BLOCK_SAMPLE_TASK => 
            timed_copy_sample(BLOCKS_DIR, 50),
        _ => {}
    }
}
