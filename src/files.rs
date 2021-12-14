use std::{fs::{self, ReadDir, File}, path::{Path, PathBuf}, string::String, io};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::{util::{log_err_none, log_err, timer, PATH_SEP}, analyze::{process_block_stream, CountedTxs}};

pub fn test_block_loads_buf(chunked_blocks_dir: &PathBuf) {
    let paths = dir_file_paths(fs::read_dir(chunked_blocks_dir).unwrap());
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

pub(crate) const BLOCKS_DIR: &str = "blocks/json";
pub fn write_json_encoded_block(slot: u64, block: &EncodedConfirmedBlock) {
    let json_r = serde_json::to_string(&block);
    match json_r {
        Ok(data) => {
            let name = slot_file_name(BLOCKS_DIR, slot, ".json");

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

pub fn load_block_json<P: AsRef<Path>>(path: P) -> Option<EncodedConfirmedBlock> {
    match fs::read(&path) {
        Ok(data) => {
            match serde_json::from_slice::<EncodedConfirmedBlock>(&data) {
                Ok(block) => Some(block),
                Err(e) => log_err_none(&e)
            }
        },
        Err(e) => {
            let r = path.as_ref();
            eprintln!("file not found:  {}", r.to_string_lossy());
            log_err_none(&e)
        }
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


const SLOT_PREFIX: &str = "slot_";
pub(crate) fn slot_json_path(slot: u64) -> String {
    format!("./{}/{}{}.json", BLOCKS_DIR, SLOT_PREFIX, slot)
}

pub(crate) fn slot_file_name(dir: &str, slot: u64, extension: &str) -> String {
    format!("{}/{}{}{}", dir, SLOT_PREFIX, slot, extension)
}


const TX_COUNT_PRE: &str = "key_tx_count_";
// TODO - should i write a trait for "save to file" ?
pub(crate) fn write_pubkey_counts(dir: &str, counts: &CountedTxs) {
    let map = counts.data;
    let path = format!("{}{}{}.json", dir, TX_COUNT_PRE, counts.total);
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
    if chunk.is_empty() { return "EMPTY".to_string() }
    // unwrap only panics if chunk is empty, so this should be safe
    let first= chunk.first().unwrap().0;
    let last = chunk.last().unwrap().0;
    chunk_json_name(first, last)
}

pub(crate) const CHUNKED_BLOCKS_DIR: &str = "blocks/json_chunked";

pub(crate) fn write_blocks_json_chunk(chunk: &Vec<SlotData>) {
    let file_name = chunk_name(chunk);

    match serde_json::to_string(chunk) {
        Ok(data) => {
            let path_str = format!("{}/{}",CHUNKED_BLOCKS_DIR, &file_name);
            let path = Path::new(&path_str);

            if Path::exists(&path) {
                println!("file {} already present, not overriding", &file_name);
            } else {
                //print!("writing block chunk file:  {}\n", p.to_str().unwrap());
                let _ = fs::write(path, data);
            }
        },
        Err(e) => log_err(&e),
    }
}

// TODO - make this a From or To trait ?
fn pathbuf_to_fname(src_path: &PathBuf) -> Option<String> {
    let src_str = src_path.to_string_lossy();
    let src_split = src_str.split(PATH_SEP);
    if let Some(fname) = src_split.last() { Some(fname.to_owned()) }
    else { None }
}


pub(crate) const BLOCK_SAMPLE_DIR: &str = "blocks/json_sample";
// copy a sample of an existing folder's files
pub(crate) fn copy_sample<P: AsRef<Path>>(path: P, one_out_of: usize) -> Result<(), std::io::Error> {
    println!("\ncopying 1 out of every {} slot_.json files to {}", one_out_of, BLOCK_SAMPLE_DIR);
    
    let read_dir = fs::read_dir(path).unwrap();
    let dir_paths = dir_file_paths(read_dir);
    let sample_size = dir_paths.len() / one_out_of;
    println!("sample size:  {}", sample_size);

    let i_range: Vec<usize> = (0..sample_size).into_iter().collect();

    i_range.par_iter().for_each(|i| {
        let src_i = i * one_out_of;
        let src_path = &dir_paths[src_i];

        let mut src = match File::open(src_path) {
            Ok(f) => f,
            Err(e) => { return log_err(&e); }       // don't stop if 1 fails, just log
        };

        let file_name = if let Some(name) = pathbuf_to_fname(&src_path) { name } 
                        else { return };
                        
        let dest_path = format!("{}/{}", BLOCK_SAMPLE_DIR, file_name);

        match File::create(dest_path) {
            Ok(mut dest) => {
                if let Err(e) = io::copy(&mut src, &mut dest) 
                    { log_err(&e) }
            }
            Err(e) => log_err(&e),
        }
    });

    Ok(())
}

pub(crate) fn timed_copy_sample<P: AsRef<Path>>(path: P, rate_arg: Option<usize>) {
    // handle default value here for now
    let rate = if let Some(r) = rate_arg { r } else { 50 };

    let elapsed = timer(|| {
        if let Err(e) = copy_sample(path, rate) { log_err(&e) }
    });
    println!("file sample copy done, time:  {:3} seconds\n", elapsed.as_secs_f32());
}


fn parse_slot_num(slot_file_name: &str) -> Option<u64> {
    let underscore_split = slot_file_name.split("_");
    let num_with_extension = underscore_split.last()?;

    let mut dot_split = num_with_extension.split(".");
    let num_str: &str = dot_split.next()?;

    match num_str.parse::<u64>() {
        Ok(n) => Some(n),
        Err(e) => { log_err(&e); None }
    }
}

pub(crate) fn slot_num_from_path(slot_path: &PathBuf) -> Option<u64> {
    match slot_path.as_os_str().to_str() {
        Some(p) => parse_slot_num(p),
        None => None,
    }
}


#[derive(Debug)]
pub(crate) struct FileSizeStats {
    pub avg: usize,
    pub count: usize
}

pub(crate) fn dir_size_stats<P: AsRef<Path>>(path: P) -> Result<FileSizeStats, std::io::Error> {
    let rd = fs::read_dir(path).unwrap();
    let file_paths = dir_file_paths(rd);
    let count = file_paths.len();

    let size_sum: usize = file_paths.par_iter().map(get_file_size).sum();
    let average: usize = size_sum/ count;

    Ok(FileSizeStats { avg: average, count: count })
}

pub(crate) fn get_file_size<P: AsRef<Path>>(path: P) -> usize {
    match fs::metadata(path) {
        Ok(meta) => meta.len() as usize,
        Err(e) => { log_err(&e); 0 },
    }
}
