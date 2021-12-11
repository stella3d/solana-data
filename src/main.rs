use crate::{
    cli::*,
    util::{hours_duration, println_each_indent}, 
    files::{
        BLOCKS_DIR,  CHUNKED_BLOCKS_DIR, 
        timed_copy_sample, test_size_average, test_block_loads, 
    },
    test_tasks::{load_perf_by_size, test_chunk_by_size},
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
        SCRAPE_BLOCKS_TASK =>
            // TODO - make network and duration (in minutes) part of cli for this command
            scrape::scrape_loop(hours_duration(12), &client::DEVNET_RPC),
        CHUNK_BLOCKS_TASK =>
            // TODO - make chunk size part of cli for this command?
            // 2 megabyte chunks tested as by far the fastest to process on my machine
            test_chunk_by_size(constants::TWO_MEGABYTES),
        COUNT_KEY_TXS_TASK => 
            test_block_loads(CHUNKED_BLOCKS_DIR),
        MEAN_FILE_SIZE_TASK => 
            test_size_average(BLOCKS_DIR),
        COMPARE_BLOCK_LOADS_TASK => 
            load_perf_by_size("blocks/sized"),
        BLOCK_SAMPLE_TASK => 
            // TODO - take sample rate as arg, maybe src directory
            timed_copy_sample(BLOCKS_DIR, 50),
        t => { 
            if t.is_empty() { eprintln!("\n--task / -t argument required!") }
            else { eprintln!("\ntask argument '{}' not recognized!", t) }
            println!("available tasks:");
            println_each_indent(&TASK_NAMES, true); 
        }
    }
}
