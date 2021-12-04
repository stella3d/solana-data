use std::{fmt::{Debug, Display}, fs::{self, ReadDir}, path::{Path, PathBuf}, result::{Result}, string::String, str::FromStr};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::util::timer;

const BLOCKS_DIR: &str = "blocks/json";

fn log_err<E: Debug + Display>(e: E) { eprintln!("{}", e); }

pub fn test_block_loads() {
    println!("\ntesting typed load of .json files...");

    let mut blocks: Vec<EncodedConfirmedBlock> = vec![];

    let par_time = timer(|| {
        blocks = load_blocks_json_par(BLOCKS_DIR);
    });
    println!("{} .json blocks loaded (parallel), {:3} seconds", blocks.len(), par_time.as_secs_f32());
    blocks.clear();

    let dir = fs::read_dir(BLOCKS_DIR).unwrap();
    let seq_time = timer(|| {
        blocks = load_blocks_json(dir);
    });
    println!("{} .json blocks loaded (sequential), {:3} seconds", blocks.len(), seq_time.as_secs_f32());
}

fn dir_file_paths(rd: ReadDir) -> Vec<PathBuf> {
    rd.map(|entry_res| {
        match entry_res {
            Ok(entry) => {
                Some(entry.path())
            },
            Err(e) => { 
                log_err(e); 
                None 
            }
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
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            }
        },
        Err(e) => {
            eprintln!("{}", e);
            None
        },
    }
}


const JSON_EXT: &str = ".json";
const SLOT_PREFIX: &str = "slot_";
fn slot_file_name(dir: &str, slot: u64, extension: &str) -> String {
    format!("./{}/{}{}{}", dir, SLOT_PREFIX, slot, extension)
}

