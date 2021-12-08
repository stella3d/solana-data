use std::{fmt::{Debug, Display}, fs::{self, ReadDir}, path::{Path, PathBuf}, result::{Result}, string::String, str::FromStr};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::{util::{timer, log_err_none}, analyze::process_block_stream};
use crate::analyze::process_blocks;

const BLOCKS_DIR: &str = "blocks/json";



pub fn test_block_loads() {
    println!("\ntesting typed load of .json files...");

    let paths = dir_file_paths(fs::read_dir(BLOCKS_DIR).unwrap());

    process_block_stream(paths.as_slice());

    /* 
    let mut blocks: Vec<EncodedConfirmedBlock> = vec![];

    let par_time = timer(|| {
        blocks = load_blocks_json_par(BLOCKS_DIR);
    });

    let ms_per = par_time.as_millis() as f64 / blocks.len() as f64;
    println!("{} .json blocks loaded (parallel): {:3} seconds, {:2} milliseconds per file", 
        blocks.len(), par_time.as_secs_f32(), ms_per);

    let sample = &blocks[..];
    process_blocks(sample);

    blocks.clear();
    */

    /* 
    let dir = fs::read_dir(BLOCKS_DIR).unwrap();
    let seq_time = timer(|| {
        blocks = load_blocks_json(dir);
    });

    let ms_per_seq = seq_time.as_millis() as f64 / blocks.len() as f64;
    println!("{} .json blocks loaded (sequential): {:3} seconds, {}ms per file", blocks.len(), seq_time.as_secs_f32(), ms_per_seq);
    */
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

pub fn write_json_encoded_block(slot: u64, block: &EncodedConfirmedBlock) {
    let json_r = serde_json::to_string(&block);
    match json_r {
        Ok(data) => {
            let name = slot_file_name("blocks/json", slot, JSON_EXT);

            if Path::exists(Path::new(&name)) {
                println!("\nFILE {} ALREADY PRESENT, not overriding", name);
            } else {
                println!("\nwriting to file:  {}", name);
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
fn slot_file_name(dir: &str, slot: u64, extension: &str) -> String {
    format!("./{}/{}{}{}", dir, SLOT_PREFIX, slot, extension)
}

