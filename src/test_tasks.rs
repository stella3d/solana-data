use std::{fs, path::{PathBuf}};

use crate::{files::{test_block_loads_buf, chunk_blocks_by_size, BLOCKS_DIR}, util::{timer, log_err}};


pub(crate) fn test_chunk_by_size(byte_count: u64) {
    println!("testing chunk by size:  {} kb per chunk max", byte_count / 1024);
    let elapsed = timer(|| {
        chunk_blocks_by_size(BLOCKS_DIR, byte_count);
    });
    println!("\nchunk by size done, time:  {:3} seconds", elapsed.as_secs_f32());
}

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