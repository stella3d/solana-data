use std::{thread, time::{Duration, Instant}, collections::HashMap, ops::Deref, path::{Path, PathBuf}, cmp::min, str::FromStr};

use rayon::iter::{ParallelIterator, IntoParallelRefIterator, IntoParallelIterator};
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::{EncodedConfirmedBlock, EncodedTransactionWithStatusMeta};

use crate::files::{load_block_json, load_block_json_unwrap, write_pubkey_counts, load_blocks_chunk_json};


pub(crate) type PubkeyTxCount = (Pubkey, u32); 
pub(crate) type PubkeyTxCountMap = HashMap<Pubkey, u32>; 
pub(crate) type PubkeyTxCountVec = Vec<(Pubkey, u32)>; 

pub(crate) struct CountedTxs<'a> {
    pub total: u32,
    pub data: &'a PubkeyTxCountVec
}

pub fn process_block_stream(block_files: &[PathBuf]) {
    println!("testing chunked stream processing...");

    let acct_set = find_account_set_stream(block_files);

    println!("done processing, converting to vec & sorting...");
    let mut accts_vec:Vec<(Pubkey, u32)> = acct_set.iter().map(|e| (*e.0, *e.1)).collect();
    accts_vec.sort_by(|e, other| e.1.cmp(&other.1));

    println!("writing account tx counts in 'blocks/'");
    write_pubkey_counts(String::from_str("blocks/").unwrap(),
        &CountedTxs { 
            total: accts_vec.len() as u32, data: &accts_vec 
        });

    thread::sleep(Duration::from_millis(10000));

    accts_vec[(accts_vec.len() - 25)..].iter().for_each(|t| {
        if t.1 > 2  {
            println!("public key: {} - entries: {}", t.0, t.1);
        }
    });

    println!("\npublic key entries: {}", acct_set.len());
}


pub fn process_blocks(blocks: &[EncodedConfirmedBlock]) {

    let single_tx_blocks: Vec<&EncodedConfirmedBlock> =
        blocks.par_iter().filter_map(|b| {
            if b.transactions.len() == 1 { Some(b) } 
            else { None }
        }).collect();

    println!("\nblocks with a single transaction (out of {}):  {}\n", blocks.len(), single_tx_blocks.len());

    thread::sleep(Duration::from_secs(2));
    
    //println!("\nprinting UI inner instructions for {} blocks:\n", blocks.len());
    blocks.par_iter().for_each(|b: &EncodedConfirmedBlock| {
        if b.transactions.len() <= 1 { return; }
        let time = b.block_time.unwrap_or_default();

        let height = if let Some(h) = b.block_height { h } else { 0 };
        let height_str = if height == 0 { "none".to_string() } else { height.to_string() };

        println!("block {}: height {}, {} txs @ {}", b.blockhash, height_str, b.transactions.len(), time);

        return;
        b.transactions.iter().for_each(|tx_meta| {
            match &tx_meta.meta {
                Some(m) => {
                    match &m.inner_instructions {
                        Some(instructions) => {
                            if instructions.len() > 0 {
                                println!("\n{} UI inner instructions:", instructions.len());
                                instructions.iter().for_each(|i| println!("    {:?}", i));
                            }
                        },
                        None => todo!(),
                    }
                },
                None => todo!(),
            };
        })
    });
}

pub fn decode_txs_map(e_txs: &Vec<EncodedTransactionWithStatusMeta>) -> Vec<Transaction> {
    e_txs.iter().filter_map(|etx| {
        if let Some(tx) = etx.transaction.decode() {
            return Some(tx)
        } else {
            return None
        }
    }).collect::<Vec<Transaction>>()
}

/* 
    given a set of paths, chunk them
    then, per chunk, serially:
    * load all files & parse into a data type 'T' (parallel)
    * run 'each_chunk' on the chunk (parallelism depends on each_chunk implementation)
    * reduce all the chunks to one single answer of the same type 'C'
*/
pub fn process_reduce_files<T, C: Send>(paths: &[PathBuf], 
    load_file: fn(&PathBuf) -> T, 
    each_chunk: fn(&[T]) -> C, 
    reduce: fn(Vec<C>) -> C) 
    -> C
{
    let chunk_size = min(100, paths.len() / 32);
    let path_chunks: Vec<&[PathBuf]> = paths.chunks(chunk_size).collect();
    println!("{} chunks of length {}", path_chunks.len(), chunk_size);

    let par_map_start = Instant::now();

    let intermediates: Vec<C> = path_chunks.par_iter()
    .map(|&chunk| {
        //let first = chunk.first().unwrap().to_str().unwrap();
        //let last = chunk.last().unwrap().to_str().unwrap();
        //println!("start chunk: {}  -  {}  @ {:?},", first, last, Instant::now());
        let typed: Vec<T> = chunk.iter().map(load_file).collect();
        each_chunk(typed.as_slice())
    }).collect();

    let reduce_start = Instant::now();
    println!("finished parallel chunked count: {:2} seconds", (reduce_start - par_map_start).as_secs_f32());

    println!("starting single-threaded reduce(), @ {:?}", reduce_start);
    
    let res = reduce(intermediates);
    
    let reduce_end = Instant::now();
    println!("finished reduce(), @ {:?}", reduce_end);
    println!("reduce() for {} elements took {}ms\n", paths.len(), (reduce_end - reduce_start).as_millis());
    res
}

pub fn process_reduce_files_chunked<T, C: Send>(paths: &[PathBuf], 
    load_chunk_file: fn(&PathBuf) -> Option<Vec<(u64, T)>>, 
    each_chunk: fn(&[(u64, T)]) -> C, 
    reduce: fn(Vec<C>) -> C) 
    -> C
{
    println!("{} chunks of length {}", paths.len(), 100);

    let par_map_start = Instant::now();

    let intermediates: Vec<C> = paths.par_iter()
    .map(|chunk_path| {
        //println!("start chunk: @ {:?},", Instant::now());
        let typed: Vec<(u64, T)> = load_chunk_file(chunk_path).unwrap();
        each_chunk(typed.as_slice())
    }).collect();

    let reduce_start = Instant::now();
    println!("finished parallel chunked count: {:2} seconds", (reduce_start - par_map_start).as_secs_f32());
    println!("starting single-threaded reduce(), @ {:?}", reduce_start);
    
    let res = reduce(intermediates);
    
    let reduce_end = Instant::now();
    println!("finished reduce(), @ {:?}", reduce_end);
    println!("reduce() for {} elements took {}ms\n", paths.len(), (reduce_end - reduce_start).as_millis());
    res
}

fn add_or_increment<T: Copy + Eq + std::hash::Hash>(key: T, hm: &mut HashMap<T, u32>) {
    match hm.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut tx_count) => {
            tx_count.insert(tx_count.get() + 1);
        },
        std::collections::hash_map::Entry::Vacant(_) => { hm.insert(key, 1); },
    };
}

const SPECIAL_ADDR_STR: &str = "11111111111111111111111111111111";

pub fn find_account_set_stream(block_files: &[PathBuf]) -> PubkeyTxCountMap {
    process_reduce_files_chunked::<EncodedConfirmedBlock, PubkeyTxCountMap>(block_files,
|p| {
            load_blocks_chunk_json(p)
        } ,
    // for each chunk, count occurences (parallel)
    find_account_set_tuple, 
    // aggregate all occurence counts (single thread, but fast)
    |sub_sets| {
        let mut outer_set = PubkeyTxCountMap::new();

        sub_sets.iter().for_each(|m| {
            //println!("chunk hashmap: {} entries", m.len());
            m.iter().for_each(|entry| {
                let pk = *entry.0;
                let sub_count = *entry.1;
                match outer_set.entry(pk) {
                    std::collections::hash_map::Entry::Occupied(mut tx_count) => {
                        tx_count.insert(tx_count.get() + sub_count);
                    },
                    std::collections::hash_map::Entry::Vacant(_) => { outer_set.insert(pk, sub_count); },
                };
            })
        });

        outer_set
    })
}

pub fn find_account_set(blocks: &[EncodedConfirmedBlock]) -> PubkeyTxCountMap {
    let mut hash_map = PubkeyTxCountMap::new();

    blocks.iter().for_each(|ecb| {
        let txs = decode_txs_map(&ecb.transactions);
        for tx in txs {
            for acct in &tx.message.account_keys {
                add_or_increment(*acct, &mut hash_map);
            }
        };
    });

    hash_map
}

pub fn find_account_set_tuple(blocks: &[(u64, EncodedConfirmedBlock)]) -> PubkeyTxCountMap {
    let mut hash_map = PubkeyTxCountMap::new();

    blocks.iter().for_each(|data| {
        let ecb = &(data.1);
        let txs = decode_txs_map(&ecb.transactions);
        for tx in txs {
            for acct in &tx.message.account_keys {
                add_or_increment(*acct, &mut hash_map);
            }
        };
    });

    hash_map
}