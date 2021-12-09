use core::fmt;
use std::{fs::{self, ReadDir}, path::{Path, PathBuf}, string::String, fmt::Display};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::{util::{log_err_none, log_err}, analyze::{process_block_stream, PubkeyTxCountMap, CountedTxs}};



pub fn test_block_loads() {
    println!("\ntesting typed load of .json files...");
    let paths = dir_file_paths(fs::read_dir(BLOCKS_DIR).unwrap());
    process_block_stream(paths.as_slice());
}

pub fn dir_file_paths(rd: ReadDir) -> Vec<PathBuf> {
    rd.map(|entry_res| {
        match entry_res {
            Ok(entry) => {
                Some(entry.path())
            },
            Err(e) => log_err_none(e) 
        }
    })
    .filter_map(|o| o)
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

pub fn load_block_json<P: AsRef<Path>>(path: P) -> Option<EncodedConfirmedBlock> {
    match fs::read(path) {
        Ok(data) => {
            match serde_json::from_slice::<EncodedConfirmedBlock>(&data) {
                Ok(block) => Some(block),
                Err(e) => log_err_none(e)
            }
        },
        Err(e) => log_err_none(e)
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
                Err(e) => log_err(e),
            }
        }
        Err(e) => { log_err(e) }
    };
}

#[derive(Debug)]
pub(crate) struct FileSizeStats {
    pub min: u64,
    pub max: u64,
    pub avg: u64,
    pub count: u64
}

pub fn test_size_average() {
    let stats = dir_size_stats(BLOCKS_DIR).unwrap();

    println!("files:\n\tcount:{}\taverage: {} bytes\n", stats.count, stats.avg)

}

pub(crate) fn dir_size_stats<P: AsRef<Path>>(path: P) -> Result<FileSizeStats, std::io::Error> {

    let rd = fs::read_dir(path).unwrap();
    let file_paths = dir_file_paths(rd);
    let count = file_paths.len() as u64;

    let size_sum: u64 = file_paths.par_iter()
        .map(get_file_size).sum();

    let average: u64 = size_sum / count as u64;

    Ok(FileSizeStats { min: 0, max: 0, avg: average, count: count })
}

pub(crate) fn get_file_size<P: AsRef<Path>>(path: P) -> u64 {
    match fs::metadata(path) {
        Ok(meta) => { meta.len() },
        Err(e) => { log_err(e); 0 },
    }
}