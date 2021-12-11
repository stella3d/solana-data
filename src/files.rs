use core::sync;
use std::{fs::{self, ReadDir, File}, path::{Path, PathBuf}, string::String, borrow::Borrow, time::{Duration, Instant}, io, cell::Ref, sync::Arc};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{de, Deserialize};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::{util::{log_err_none, log_err}, analyze::{process_block_stream, CountedTxs}};



pub fn test_block_loads() {
    println!("\ntesting CHUNKED typed load of .json files...");
    let paths = dir_file_paths(fs::read_dir(CHUNKED_BLOCKS_DIR).unwrap());
    process_block_stream(paths.as_slice());
}

pub fn dir_file_paths(rd: ReadDir) -> Vec<PathBuf> {
    rd.map(|entry_res| {
        match entry_res {
            Ok(entry) => {
                Some(entry.path())
            },
            Err(e) => log_err_none(&e) 
        }
    })
    .filter_map(|o| o)
    .collect()
}

fn dir_file_names(rd: ReadDir) -> Vec<PathBuf> {
    rd.map(|entry_res| {
        match entry_res {
            Ok(entry) => {
                Some(entry.file_name())
            },
            Err(e) => log_err_none(&e) 
        }
    })
    .map(|o| {
        PathBuf::from(o.unwrap())
    })
    .collect()
}

pub fn load_blocks_json_par<P: AsRef<Path>>(dir: P) -> Vec<EncodedConfirmedBlock> {
    let rd = fs::read_dir(dir).unwrap();
    let file_paths = dir_file_paths(rd);
    println!("{} files in dir", file_paths.len());
    // not using full paths might break later, but easy to fix
    file_paths.par_iter().map(load_block_json)
    .filter_map(|opt| opt)
    .collect()
}

// assumes every file in folder is a blocks json
pub fn load_blocks_json(dir: ReadDir) -> Vec<EncodedConfirmedBlock> {
    let mut blocks = Vec::<EncodedConfirmedBlock>::new();

    dir.into_iter().for_each(|result| {
        match result {
            Ok(entry) => {
                match load_block_json(entry.path()) {
                    Some(b) => blocks.push(b),
                    None => println!("FAILED to load block .json:  {}", entry.file_name().to_str().unwrap()),
                }
            },
            Err(e) => eprintln!("{}", e),
        }
    });
    blocks
}

pub(crate) const BLOCKS_DIR: &str = "blocks/json";
pub fn write_json_encoded_block(slot: u64, block: &EncodedConfirmedBlock) {
    let json_r = serde_json::to_string(&block);
    match json_r {
        Ok(data) => {
            let name = slot_file_name(BLOCKS_DIR, slot, JSON_EXT);

            if Path::exists(Path::new(&name)) {
                println!("FILE {} ALREADY PRESENT, not overriding", name);
            } else {
                print!("writing file:  {}\n", name);
                let _ = fs::write(name, data);
            }
        },
        Err(e) => eprintln!("{}", e),
    }
}

/*
pub fn load_blocks_generic<'a, P: AsRef<Path>, T: Deserialize<'a>>(path: P) -> Option<T> {
    let data = fs::read(path).unwrap_or(vec![]);
    let res: Option<T> = parse_blocks_gen(&data);
    res
}

pub fn parse_blocks_gen<'a, T: Deserialize<'a>>(buffer: &'a Vec<u8>) -> Option<T> {
    let b: &'a Vec<u8> = buffer;
    match serde_json::from_slice::<'a, T>(b) {
        Ok(block) => Some(block),
        Err(e) => log_err_none(e)
    }
}
*/

pub fn load_block_json<P: AsRef<Path>>(path: P) -> Option<EncodedConfirmedBlock> {
    match fs::read(path) {
        Ok(data) => {
            match serde_json::from_slice::<EncodedConfirmedBlock>(&data) {
                Ok(block) => Some(block),
                Err(e) => log_err_none(&e)
            }
        },
        Err(e) => log_err_none(&e)
    }
}

pub fn load_blocks_chunk_json<P: AsRef<Path>>(path: P) -> Option<Vec<SlotData>> {
    match fs::read(path) {
        Ok(data) => {
            match serde_json::from_slice::<Vec<SlotData>>(&data) {
                Ok(slots) => Some(slots),
                Err(e) => log_err_none(&e)
            }
        },
        Err(e) => log_err_none(&e)
    }
}

pub fn load_block_json_unwrap<P: AsRef<Path>>(path: P) -> EncodedConfirmedBlock {
    load_block_json(path).unwrap()
}

const JSON_EXT: &str = ".json";
const SLOT_PREFIX: &str = "slot_";

pub(crate) fn slot_json_path(slot: u64) -> String {
    format!("./{}/{}{}{}", BLOCKS_DIR, SLOT_PREFIX, slot, JSON_EXT)
}

pub(crate) fn slot_file_name(dir: &str, slot: u64, extension: &str) -> String {
    format!("{}/{}{}{}", dir, SLOT_PREFIX, slot, extension)
}

const TX_COUNT_PRE: &str = "key_tx_count_";
// TODO - should i write a trait for "save to file" ?
pub(crate) fn write_pubkey_counts(dir: String, counts: &CountedTxs) {
    let map = counts.data;
    let path = format!("{}{}{}{}", dir, TX_COUNT_PRE, counts.total, JSON_EXT);
    match serde_json::to_string(&map) {
        Ok(json) => {
            match fs::write(&path, json) {
                Ok(_) => println!("{} written", path),
                Err(e) => log_err(&e),
            }
        }
        Err(e) => { log_err(&e) }
    };
}

pub(crate) type SlotData = (u64, EncodedConfirmedBlock);

pub(crate) fn chunk_json_name(first: u64, last: u64) -> String {
    format!("slots_{}-{}.json", first, last)
}

pub(crate) fn chunk_name(chunk: &Vec<SlotData>) -> String {
    let first= chunk.first().unwrap().0;
    let last = chunk.last().unwrap().0;
    chunk_json_name(first, last)
}

pub(crate) fn chunk_path_from_inputs(chunk: &[PathBuf]) -> String {
    let f_path: &PathBuf = chunk.first().unwrap();
    let first = slot_num_from_path(f_path).unwrap();
    let l_path: &PathBuf = chunk.last().unwrap();
    let last = slot_num_from_path(l_path).unwrap();

    let file_name = chunk_json_name(first, last);

    format!("{}/{}",CHUNKED_BLOCKS_DIR, &file_name).to_string()
}

pub(crate) fn write_blocks_json_chunk(chunk: &Vec<SlotData>) {
    let file_name = chunk_name(chunk);

    match serde_json::to_string(chunk) {
        Ok(data) => {
            let f_str = &("/".to_string() + &file_name);
            let p_str = format!("{}{}",CHUNKED_BLOCKS_DIR, f_str);
            let p = Path::new(&p_str);

            if Path::exists(&p) {
                println!("file {} already present, not overriding", &file_name);
            } else {
                //print!("writing block chunk file:  {}\n", p.to_str().unwrap());
                let _ = fs::write(p, data);
            }
        },
        Err(e) => log_err(&e),
    }
}

pub(crate) const BLOCK_SAMPLE_DIR: &str = "blocks\\json_sample";
pub(crate) fn copy_sample<P: AsRef<Path>>(path: P, one_out_of: usize) -> Result<(), std::io::Error> {
    println!("\ncopying 1 out of every {} slot_.json files to {}", one_out_of, BLOCK_SAMPLE_DIR);

    let read_dir = fs::read_dir(path).unwrap();
    let dir_paths = dir_file_paths(read_dir);
    let sample_size = dir_paths.len() / one_out_of;

    let i_range: Vec<usize> = (0..sample_size).into_iter().collect();

    i_range.par_iter().for_each(|i| {
        let src_i = i * one_out_of;
        let src_path = &dir_paths[src_i];
        println!("copy to sample:  {:?}", &src_path);

        let mut src_file = File::open(src_path).unwrap();
        let src_str = src_path.to_string_lossy();
        let src_split = src_str.split("\\");

        let last_split = src_split.last().unwrap();
        //println!("last split:  {:?}", &last_split);

        let dest_path = format!("{}/{}", BLOCK_SAMPLE_DIR, last_split);
        //println!("dest path:  {:?}", &dest_path);

        match File::create(dest_path) {
            Ok(mut dest_file) => {
                match io::copy(&mut src_file, &mut dest_file) {
                    Ok(_) => {},
                    Err(e) => { log_err(&e); },
                };
            },
            Err(e) => { log_err(&e); },
        }
    });

    Ok(())
}

fn parse_slot_num(slot_file_name: &str) -> Option<u64> {
    let underscore_split = slot_file_name.split("_");
    let num_with_extension = underscore_split.last()?;

    let mut dot_split = num_with_extension.split(".");
    let num_str: &str = dot_split.next()?;

    //println!("parsing file: {},  number:  {}", slot_file_name, num_str);
    match num_str.parse::<u64>() {
        Ok(n) => Some(n),
        Err(e) => { log_err(&e); None }
    }
}

fn slot_num_from_path(slot_path: &PathBuf) -> Option<u64> {
    match slot_path.as_os_str().to_str() {
        Some(p) => parse_slot_num(p),
        None => None,
    }
}

pub(crate) const CHUNKED_BLOCKS_DIR: &str = "blocks/json_chunked";

pub(crate) type SizedPath<'a> = (&'a PathBuf, u64);

pub(crate) fn test_chunk_by_size(byte_count: u64) {
    println!("testing chunking by size:  {} bytes max per chunk", byte_count);
    chunk_blocks_by_size(BLOCKS_DIR, byte_count);
}

pub(crate) fn chunk_blocks_by_size(blocks_dir: &str, max_input_bytes: u64) {
    let src_paths = dir_file_paths(fs::read_dir(blocks_dir).unwrap());
    let src_sizes: Vec<SizedPath> = src_paths.par_iter()
        .map(|p| (p, get_file_size(p)))
        .collect();

    const chunk_count: usize = 24;
    let chunk_len = src_sizes.len() / chunk_count;
    let sizes_chunks: Vec<&[SizedPath]> = src_sizes.chunks(chunk_len).collect();

    let input_path_chunks: Vec<Arc<Vec<&PathBuf>>> = sizes_chunks
        .par_iter()
        .flat_map(|&chunk| {
            let mut chunk_outputs = Vec::<Arc<Vec<&PathBuf>>>::new();
            let mut data = Vec::<&PathBuf>::new();

            let mut size_counter: u64 = 0;
            chunk.iter().for_each(|&sp| {
                let size = sp.1;
                if size_counter + size > max_input_bytes {
                    println!("creating input chunk:  {} bytes", size_counter);
                    chunk_outputs.push(Arc::new(data));
                    data = Vec::<&PathBuf>::new();
                    size_counter = 0;
                }
                
                size_counter += size;
                data.push(sp.0);
            });

            chunk_outputs
        })
        .collect();


    let dir_path = get_blocks_dir();
    input_path_chunks.par_iter().for_each(|chunk| {
        let slot_data: Vec<SlotData> = chunk.into_iter()
        .filter_map(|name| {
            let full_path = dir_path.join(name);
            match load_block_json(full_path) {
                Some(ecb) => {
                    let as_str = name.as_os_str().to_str()?;
                    match parse_slot_num(as_str) {
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
}

fn get_blocks_dir<'a>() -> &'a Path {
    let mut dir_str = BLOCKS_DIR.to_owned();
    dir_str.push_str("/");
    Path::new(BLOCKS_DIR)
}

pub(crate) fn chunk_existing_blocks(chunk_len: usize) {
    println!("\ncopy existing single block files to {} block chunks...\n", chunk_len);

    let start_time = Instant::now();
    let src_names = dir_file_names(fs::read_dir(BLOCKS_DIR).unwrap());
    let chunks: Vec<&[PathBuf]> = src_names.chunks(chunk_len).collect();

    let mut dir_str = BLOCKS_DIR.to_owned();
    dir_str.push_str("/");
    let dir_path = Path::new(BLOCKS_DIR);

    chunks.par_iter().for_each(|&chunk| {
        let c_path_str = &chunk_path_from_inputs(&chunk);
        let chunk_out_path = Path::new(c_path_str);
        if Path::exists(chunk_out_path) { 
            println!("skipping pre-existing chunk:  {}", chunk_out_path.to_string_lossy());
            return 
        }

        let slot_data: Vec<SlotData> = chunk.into_iter()
            .filter_map(|name| {
                let full_path = dir_path.join(name);
                match load_block_json(full_path) {
                    Some(ecb) => {
                        let as_str = name.as_os_str().to_str()?;
                        match parse_slot_num(as_str) {
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

    let end_time = Instant::now();
    let elapsed: Duration = end_time - start_time;
    println!("\nfinished writing {} chunks of blocks, time:  {}ms\n", chunks.len(), elapsed.as_millis());
}


#[derive(Debug)]
pub(crate) struct FileSizeStats {
    pub avg: u64,
    pub count: u64
}

pub fn test_size_average(dir: &str) {
    let stats = dir_size_stats(dir).unwrap();
    println!("files:\n\tcount:{}\taverage: {} kb\n", stats.count, stats.avg / 1024)
}

pub(crate) fn dir_size_stats<P: AsRef<Path>>(path: P) -> Result<FileSizeStats, std::io::Error> {
    let rd = fs::read_dir(path).unwrap();
    let file_paths = dir_file_paths(rd);
    let count = file_paths.len() as u64;

    let size_sum: u64 = file_paths.par_iter().map(get_file_size).sum();
    let average: u64 = size_sum / count as u64;

    Ok(FileSizeStats { avg: average, count: count })
}

pub(crate) fn get_file_size<P: AsRef<Path>>(path: P) -> u64 {
    match fs::metadata(path) {
        Ok(meta) => meta.len(),
        Err(e) => { log_err(&e); 0 },
    }
}