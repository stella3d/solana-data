use std::{fs::{read_dir, ReadDir}, path::PathBuf, process::exit, fmt::{Debug, Display}};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    cli::CliArguments,
    util::{timer, log_err, MEGABYTE}, 
    files::{
        BLOCKS_DIR, SlotData, dir_file_paths, get_file_size, 
        slot_num_from_path, load_block_json, write_blocks_json_chunk
    } 
};


type SizedPath<'a> = (&'a PathBuf, usize);      // file's path + size in bytes


// run a task that we can't proceed without the success of, exit if it fails
pub(crate) fn do_or_exit<F: FnOnce() -> Result<T, E>, T, E: Debug + Display>
    (task: F, err_msg: &str) -> T 
{
    match task() {
        Ok(output) => output,
        Err(e) => {
            log_err(&e);
            log_err(err_msg);
            exit(1);
        }}
}


pub(crate) fn chunk_blocks_by_size(src_dir: ReadDir, max_input_bytes: usize) {
    let src_paths = dir_file_paths(src_dir);
    let src_sizes: Vec<SizedPath> = src_paths
        .par_iter()
        .map(|p| (p, get_file_size(p)))
        .collect();

    println!("got source file sizes, count:  {}", src_sizes.len());

    // magic number 24 should probably be number of threads available
    const TASK_COUNT: usize = 24;
    let task_len = src_sizes.len() / TASK_COUNT;
    let sizes_chunks: Vec<&[SizedPath]> = src_sizes.chunks(task_len).collect();

    // given input dir of single-block files, get sequential groups of
    // input paths that each total close to the chunk size limit.
    let input_path_chunks: Vec<Vec<&PathBuf>> = sizes_chunks
        .par_iter()
        .flat_map(|&input_slice| {
            let mut chunk_outputs = Vec::<Vec<&PathBuf>>::new();
            let mut data = Vec::<&PathBuf>::new();

            let mut size_count: usize = 0;
            input_slice.iter().for_each(|&sized_path| {
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
        })
        .collect();
    
    println!("got vec<vec<PathBuf>> of input paths, len:  {}", input_path_chunks.len());

    input_path_chunks.par_iter().for_each(|chunk| {
        // given the chunk of input paths, load and parse them, discarding any that don't parse.
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

        // after a chunk is collected, save it to a file 
        write_blocks_json_chunk(&slot_data);
    });

    println!("done running:  chunk_blocks_by_size()");
}


const NO_DIR_EXIT_MSG: &str = "can't proceed without a valid directory!\nexiting\n";

// handler for the 'chunk_blocks' CLI task
pub(crate) fn chunk_by_size_cli(args: &CliArguments) {
    let size = if let Some(mbs) = args.chunk_size { MEGABYTE * mbs }
               else { MEGABYTE * 2 };       // 2mb benchmarked best on dev machine, so it's default
    
    // TODO - make hardcoded BLOCKS_DIR path into CLI arg
    // exit if source can't be read
    let src_dir = do_or_exit(|| read_dir(BLOCKS_DIR), NO_DIR_EXIT_MSG);

    println!("chunking blocks by size:  {} kb per sequential group, max", size / 1024);
    let elapsed = timer(|| {
        chunk_blocks_by_size(src_dir, size);
    });
    println!("done, time:  {:3} seconds", elapsed.as_secs_f32());
}