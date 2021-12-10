use core::{fmt, num};
use std::{fs::{self, ReadDir}, path::{Path, PathBuf}, string::String, fmt::Display, str::FromStr};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::{util::{log_err_none, log_err}, analyze::{process_block_stream, PubkeyTxCountMap, CountedTxs}};



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
            Err(e) => log_err_none(e) 
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
            Err(e) => log_err_none(e) 
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

pub fn load_blocks_chunk_json<P: AsRef<Path>>(path: P) -> Option<Vec<SlotData>> {
    match fs::read(path) {
        Ok(data) => {
            match serde_json::from_slice::<Vec<SlotData>>(&data) {
                Ok(slots) => Some(slots),
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

pub(crate) type SlotData = (u64, EncodedConfirmedBlock);

pub(crate) fn write_blocks_json_chunk(chunk: &Vec<SlotData>) {
    let f = chunk.first();
    let first = f.unwrap().0;

    let l = chunk.last();
    let last = l.unwrap().0;
    let file_name =  format!("slots_{}-{}.json", first, last);

    let json_r = serde_json::to_string(chunk);

    match json_r {
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
        Err(e) => log_err(e),
    }
}

fn parse_slot_num(slot_file_name: &str) -> Option<u64> {
    let underscore_split = slot_file_name.split("_");
    let num_with_extension = underscore_split.last()?;

    let mut dot_split = num_with_extension.split(".");
    let num_str: &str = dot_split.next()?;

    //println!("parsing file: {},  number:  {}", slot_file_name, num_str);
    match num_str.parse::<u64>() {
        Ok(n) => Some(n),
        Err(e) => { log_err(e); None }
    }
}

pub(crate) const CHUNKED_BLOCKS_DIR: &str = "blocks/json_chunked";

pub(crate) fn chunk_existing_blocks(chunk_len: usize) {
    println!("\ncopy existing single block files to {} block chunks...\n", chunk_len);

    let src_names = dir_file_names(fs::read_dir(BLOCKS_DIR).unwrap());
    let chunks: Vec<&[PathBuf]> = src_names.chunks(chunk_len).collect();

    let mut dir_str = BLOCKS_DIR.to_owned();
    dir_str.push_str("/");
    let dir_path = Path::new(BLOCKS_DIR);

    chunks.par_iter().for_each(|&chunk| {
        //println!("start chunk");
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


#[derive(Debug)]
pub(crate) struct FileSizeStats {
    pub min: u64,
    pub max: u64,
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