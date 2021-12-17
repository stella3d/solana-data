use std::{fs::{self, read_dir}, path::{PathBuf}, time::{Instant, Duration}, thread, str::FromStr};

use solana_client::rpc_config::{RpcAccountInfoConfig, RpcTransactionLogsFilter, RpcTransactionLogsConfig};
use solana_program::pubkey::Pubkey;

use crate::{
    analyze::process_block_stream, client::SolClient,
    files::{test_block_loads_buf, CHUNKED_BLOCKS_DIR, dir_file_paths, dir_size_stats}, 
    util::{log_err, timer, ok_or_die} 
};


// load multiple folders of files, containing the same source data 
// grouped into different size chunks, compare how performance varies with size
pub(crate) fn load_perf_by_size(chunked_data_dir: &str) {
    let dir = ok_or_die(|| read_dir(chunked_data_dir));

    dir.into_iter().for_each(|dir_entry| {
        match dir_entry {
            Ok(de) => {
                let path: PathBuf = de.path();
                let path_str = path.to_string_lossy();
                if !path.exists() {
                    eprintln!("directory {} not found!", path_str);
                }

                println!("loading data dir:  {}", path_str);
                let elapsed = timer(|| { 
                    test_block_loads_buf(&path); 
                });
                println!("finished load & process in {:3} seconds\n", elapsed.as_secs_f32());
            },
            Err(e) => log_err(&e),
        }
    });
}

pub(crate) fn test_block_loads(chunked_blocks_dir: &str) {
    let mut dir = chunked_blocks_dir;
    if dir.is_empty() { dir = CHUNKED_BLOCKS_DIR }

    println!("\nloading + processing chunked Solana block data from {}", dir);
    match fs::read_dir(dir) {
        Ok(rd) => {
            let paths = dir_file_paths(rd);
            process_block_stream(paths.as_slice());
        },
        Err(e) => log_err(&e)
    };
}

// just see if the average file size code runs
pub fn test_size_average(dir: &str) {
    let stats = match dir_size_stats(dir) {
        Ok(ss) => ss,
        Err(e) => return log_err(&e)
    };
    println!("files:\n\tcount:{}\taverage: {} kb\n", stats.count, stats.avg / 1024);
}

pub(crate) fn test_get_block_production(client: &SolClient, logging: bool) {
    if let Ok(prod) = client.get_block_production() {
        if logging { 
            let first = prod.range.first_slot;
            let last = prod.range.last_slot;
            println!("\nepoch slot range:  {} - {}", first, last);

            prod.by_identity.iter().for_each(|id| {
                println!("slot leader pubkey:  {}", id.0);
                println!("        this epoch:  lead {} slots, produced {} blocks", id.1.0, id.1.1);
            });
        }
        // TODO - more here
    }
}

pub(crate) fn test_basic_subscriptions(client: &mut SolClient) {
    println!("rpc websocket url:  {}", &client.ws_url);
    match client.slot_subscribe() {
        Ok(_) => { println!("slot_subscribe() success") },
        Err(e) => log_err(&e),
    };
    match client.logs_subscribe(RpcTransactionLogsFilter::All, RpcTransactionLogsConfig { commitment: None } ) {
        Ok(_) => { println!("logs_subscribe() success") },
        Err(e) => log_err(&e),
    };

    let subscribe_to_prog = Pubkey::from_str("2yqG9bzKHD59MxD9q7ExLvnDhNycB3wkvKXFQSpBoiaE").unwrap();
    match client.program_subscribe(&subscribe_to_prog) {
        Ok(_) => { println!("program_subscribe() success") },
        Err(e) => log_err(&e),
    }

    let ms10 = Duration::from_millis(10);
    let mins = Duration::from_millis(1000 * 60 * 5);
    let start = Instant::now();
    while Instant::now() - start < mins {
        client.try_recv_all();
        thread::sleep(ms10);
    }

    println!("end test_basic_subscriptions()");
}