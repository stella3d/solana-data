use std::{fs::{self, ReadDir, File}, path::{Path, PathBuf}, string::String, io};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use solana_transaction_status::EncodedConfirmedBlock;
use serde_json;

use crate::{util::{log_err_none, log_err, timer}, analyze::{process_block_stream, CountedTxs}};

pub fn test_block_loads_buf(chunked_blocks_dir: &PathBuf) {
    println!("\ntesting chunked, typed load of .json files...");
    let paths = dir_file_paths(fs::read_dir(chunked_blocks_dir).unwrap());
    process_block_stream(paths.as_slice());
}

pub fn test_block_loads(chunked_blocks_dir: &str) {
    let mut dir = chunked_blocks_dir;
    if dir.is_empty() { dir = CHUNKED_BLOCKS_DIR }

    println!("\ntesting load + process of blocks .json files...");
    match fs::read_dir(dir) {
        Ok(rd) => {
            let paths = dir_file_paths(rd);
            process_block_stream(paths.as_slice());
        },
        Err(e) => log_err(&e)
    };
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

pub(crate) const BLOCK_SAMPLE_DIR: &str = "blocks/json_sample";
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
        //println!("copy to sample:  {:?}", &src_path);

        let mut src = File::open(src_path).unwrap();
        let src_str = src_path.to_string_lossy();
        let src_split = src_str.split("\\");
        let last_split = src_split.last().unwrap();
        let dest_path = format!("{}/{}", BLOCK_SAMPLE_DIR, last_split);

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

pub(crate) const CHUNKED_BLOCKS_DIR: &str = "blocks/json_chunked";

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
