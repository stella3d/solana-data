use std::{fs, path::PathBuf};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{files::{BLOCKS_DIR, dir_file_paths, get_file_size, SlotData, load_block_json, slot_num_from_path, write_blocks_json_chunk}, util::timer, cli::{chunk_size_or_default, CliArguments}};


type SizedPath<'a> = (&'a PathBuf, u64);            // file's path + size in bytes

pub(crate) fn chunk_by_size(byte_count: u64) {
    println!("chunk by size:  {} kb per chunk max", byte_count / 1024);
    let elapsed = timer(|| {
        chunk_blocks_by_size(BLOCKS_DIR, byte_count);
    });
    println!("\nchunk by size done, time:  {:3} seconds", elapsed.as_secs_f32());
}

pub(crate) fn chunk_by_size_cli(args: &CliArguments) {
    // TODO - better pattern for argument defaults
    let byte_count = chunk_size_or_default(&args);
    chunk_by_size(byte_count as u64);
}

pub(crate) fn chunk_blocks_by_size(blocks_dir: &str, max_input_bytes: u64) {
    let src_paths = dir_file_paths(fs::read_dir(blocks_dir).unwrap());
    let src_sizes: Vec<SizedPath> = src_paths.par_iter()
        .map(|p| (p, get_file_size(p)))
        .collect();

    println!("got source file sizes, count:  {}", src_sizes.len());

    const TASK_COUNT: usize = 24;
    let task_len = src_sizes.len() / TASK_COUNT;
    let sizes_chunks: Vec<&[SizedPath]> = src_sizes.chunks(task_len).collect();

    let input_path_chunks: Vec<Vec<&PathBuf>> = sizes_chunks
        .par_iter()
        .flat_map(|&chunk| {
            let mut chunk_outputs = Vec::<Vec<&PathBuf>>::new();
            let mut data = Vec::<&PathBuf>::new();

            let mut size_count: u64 = 0;
            chunk.iter().for_each(|&sized_path| {
                let path = sized_path.0;
                let size = sized_path.1;

                let mut pushed = false;
                if size > max_input_bytes && size_count == 0 {
                    // this one block is bigger than our target chunk size, make a chunk of 1
                    println!("single block chunk:  {} bytes", size);
                    data.push(path);
                    size_count += size;
                    pushed = true;
                }
                if pushed || size_count + size > max_input_bytes {
                    println!("creating input chunk:  {} bytes,  {} blocks", size_count, data.len());
                    chunk_outputs.push(data.clone());
                    data.clear();
                    size_count = 0;
                }
                if !pushed { 
                    data.push(path); 
                    size_count += size;
                }
            });

            chunk_outputs
        }).collect();
    
    println!("got vec<vec<PathBuf>> of input paths, len:  {}", input_path_chunks.len());

    input_path_chunks.par_iter().for_each(|chunk| {
        let slot_data: Vec<SlotData> = chunk.iter()
        .filter_map(|&path| {
            match load_block_json(path) {
                Some(ecb) => {
                    match slot_num_from_path(path) {
                        Some(num) => Some((num, ecb)),
                        None => None,
                    }
                },
                None => None,
            }
        })
        .collect(); 

        write_blocks_json_chunk(&slot_data);
    });
    println!("done running:  chunk_blocks_by_size()");
}