use crate::{
    cli::*,
    util::{minutes_duration, println_each_indent}, 
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
pub mod constants;
 
fn main() {
    let cli_args = get_cli_args();
    
    match cli_args.task.as_str() {
        // TODO - make network/rpc part of cli for this command
        SCRAPE_BLOCKS_TASK => {
            let mins = cli_args.minutes.unwrap_or(60);
            let duration = minutes_duration(mins);
            println!("\nscraping blocks from RPC for {} minutes\n", mins);
            scrape::scrape_loop(duration, &client::DEVNET_RPC);
        },
        // TODO - make chunk size part of cli for this command?
        CHUNK_BLOCKS_TASK =>
            // 2mb chunks tested by far the fastest to process on my machine
            test_chunk_by_size(constants::TWO_MEGABYTES),
        COUNT_KEY_TXS_TASK => 
            test_block_loads(CHUNKED_BLOCKS_DIR),
        MEAN_FILE_SIZE_TASK => 
            test_size_average(BLOCKS_DIR),
        COMPARE_BLOCK_LOADS_TASK => 
            load_perf_by_size("blocks/sized"),
        // TODO - take sample rate as arg, maybe src directory
        BLOCK_SAMPLE_TASK => 
            timed_copy_sample(BLOCKS_DIR, 50),
        t => { 
            if t.is_empty() { eprintln!("\n--task / -t argument required!") }
            else { eprintln!("\ntask argument '{}' not recognized!", t) }
            println!("available tasks:");
            println_each_indent(&TASK_NAMES, true); 
        }
    }
}
