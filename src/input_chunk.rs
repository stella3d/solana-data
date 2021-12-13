use std::{fs::{read_dir, ReadDir}, path::{PathBuf}};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    cli::CliArguments,
    util::{timer, MEGABYTE, do_or_die}, 
    files::{
        BLOCKS_DIR, SlotData, dir_file_paths, get_file_size, 
        slot_num_from_path, load_block_json, write_blocks_json_chunk
    } 
};


type SizedPath<'a> = (&'a PathBuf, usize);      // file's path + size in bytes


// given a dir of many single-block .json files, group the inputs sequentially,  
// each group sized as close to the limit as possible.
// parse those groups, write them to single files in the out dir
pub(crate) fn chunk_blocks_by_size(src_dir: ReadDir, max_input_bytes: usize) {
    let src_paths = dir_file_paths(src_dir);
    let src_sizes: Vec<SizedPath> = src_paths
        .par_iter()
        .map(|p| (p, get_file_size(p)))
        .collect();

    println!("source file count:  {}", src_sizes.len());

    // limited number of tasks reduces undersized chunks without much drawback - usually disk bound
    // actual count is TASK_COUNT+1, because of the remainder chunk
    const TASK_COUNT: usize = 8;
    let task_len = src_sizes.len() / TASK_COUNT;
    println!("task length:  {}", task_len);

    let sizes_chunks: Vec<&[SizedPath]> = src_sizes.chunks(task_len).collect();
    println!("input slice count:  {}", sizes_chunks.len());

    // parallelizes, with a minor issue: the last chunk in each slice being undersized
    // slice often runs out of src paths before full chunk accrues
    // almost always enough output files that this is not an issue
    let input_path_chunks: Vec<Vec<&PathBuf>> = sizes_chunks
        .par_iter()
        .flat_map(|&paths| sized_path_chunks(&paths, max_input_bytes))
        .collect();
    
    println!("output chunk count:  {}", input_path_chunks.len());

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
}

// get sequential groups of input paths that each total as close to the size limit as possible.
fn sized_path_chunks<'a>(inputs: &'a [SizedPath], max_bytes: usize) -> Vec<Vec<&'a PathBuf>> {
    let mut chunk_outputs = Vec::<Vec<&PathBuf>>::new();
    let mut data = Vec::<&PathBuf>::new();

    // TODO - handle instead of unwrap ? not sure how last() can fail
    let last_input = inputs.last().unwrap();
    
    let mut size_count: usize = 0;
    inputs.iter().for_each(|sized_path| {
        let path = sized_path.0;
        let size = sized_path.1;
        let mut pushed = false;

        if size > max_bytes && size_count == 0 {
            // this 1 block is bigger than our target chunk size, make a chunk of 1
            println!("single block chunk:  {} bytes", size);
            data.push(path);
            size_count += size;
            pushed = true;
        }
        
        let next_size = size_count + size;
        if next_size < max_bytes && path == last_input.0 {
            // println!("pushing undersize chunk from end of iterator");
            data.push(path);
            size_count += size;
            pushed = true;
        }

        let chunk_ready = next_size >= max_bytes;
        if pushed || chunk_ready {
            //println!("output chunk:  {} bytes,  {} blocks", size_count, data.len());
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
}

const NO_DIR_EXIT_MSG: &str = "can't proceed without a valid directory!\nexiting\n";

// handler for the 'chunk_blocks' CLI task
pub(crate) fn chunk_by_size_cli(args: &CliArguments) {
    let size = if let Some(mbs) = args.chunk_size { MEGABYTE * mbs }
               else { MEGABYTE * 2 };       // 2mb benchmarked best on dev machine, so it's default
    
    // TODO - make hardcoded BLOCKS_DIR path into CLI arg
    // exit if source can't be read
    let src_dir = do_or_die(|| read_dir(BLOCKS_DIR), NO_DIR_EXIT_MSG);

    println!("chunking blocks by size:  {} kb per sequential group, max", size / 1024);
    let elapsed = timer(|| {
        chunk_blocks_by_size(src_dir, size);
    });
    println!("done, time:  {:3} seconds", elapsed.as_secs_f32());
}