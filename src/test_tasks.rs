use std::{fs, path::{PathBuf}};

use crate::{files::{test_block_loads_buf, CHUNKED_BLOCKS_DIR, dir_file_paths, dir_size_stats}, util::{log_err, timer}, analyze::process_block_stream, client::ClientWrapper};


// load multiple folders of files, containing the same source data 
// grouped into different size chunks, compare how performance varies with size
pub(crate) fn load_perf_by_size(chunked_data_dir: &str) {
    println!("\nstart load test on data dir:\n\t{}", chunked_data_dir);

    match fs::read_dir(&chunked_data_dir) {
        Ok(rd) => {
            rd.into_iter().for_each(|dir_entry| {
                match dir_entry {
                    Ok(de) => {
                        let path: PathBuf = de.path();
                        let path_str = path.to_string_lossy();
                        if !path.exists() {
                            eprintln!("directory {} not found!", path_str);
                        }
                        println!("running load test on chunked data dir:\n\t{}\n", path_str);
                        let elapsed = timer(|| { 
                            test_block_loads_buf(&path); 
                        });
                        println!("LOAD TIME:  {:3} seconds\n", elapsed.as_secs_f32());
                    },
                    Err(e) => log_err(&e),
                }
            });
        },
        Err(e) => log_err(&e),
    };
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
    let stats = dir_size_stats(dir).unwrap();
    println!("files:\n\tcount:{}\taverage: {} kb\n", stats.count, stats.avg / 1024)
}

pub(crate) fn test_get_block_production(client: &ClientWrapper, logging: bool) {
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
    }
}