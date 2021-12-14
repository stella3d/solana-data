use std::{collections::HashMap, path::{PathBuf}};

use rayon::iter::{ParallelIterator, IntoParallelRefIterator};
use solana_program::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::{EncodedConfirmedBlock, EncodedTransactionWithStatusMeta};

use crate::{files::{write_pubkey_counts, load_blocks_chunk_json}, util::{time_run, log_err}};


pub(crate) type PubkeyTxCount = (Pubkey, u32); 
pub(crate) type PubkeyTxCountMap = HashMap<Pubkey, u32>; 

pub(crate) struct CountedTxs<'a> {
    pub total: u32,
    pub data: &'a Vec<PubkeyTxCount>
}

pub fn process_block_stream(block_files: &[PathBuf]) {
    println!("testing chunked stream processing...");

    let acct_set = find_account_set_stream(block_files);

    println!("done processing, converting to vec & sorting...");
    let mut accts_vec:Vec<(Pubkey, u32)> = acct_set.iter().map(|e| (*e.0, *e.1)).collect();
    accts_vec.sort_by(|e, other| e.1.cmp(&other.1));

    println!("writing account tx counts in 'blocks/'");
    let key_counts = &CountedTxs { total: accts_vec.len() as u32, data: &accts_vec };
    write_pubkey_counts("blocks/", key_counts);
        
    println!("\nunique public keys counted: {}\n", acct_set.len());

    accts_vec[(accts_vec.len() - 15)..].iter().for_each(|t| {
        println!("public key: {} - entries:  {}", t.0, t.1);
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
    given a set of paths pointing to data parsable as 'T':
    * load all files & parse to type 'T'
    * run 'each_chunk' on data parsed from each file
    * reduce all the chunks' output to one single value of the same type, 'C'
*/
pub fn map_reduce_chunk_files<U: Sized + Send, T: Send, C: Send>(
    paths: &[PathBuf], 
    load_chunk_file: fn(&PathBuf) -> Option<Vec<(U, T)>>, 
    each_chunk: fn(&[(U, T)]) -> C, 
    reduce: fn(Vec<C>) -> C) 
    -> C
{
    // in parallel, load & parse each chunk, before running supplied fn on the data
    let sub_results: Vec<C> = paths.par_iter()
    .filter_map(|path| {
        match load_chunk_file(path) {
            Some(c) => Some(each_chunk(&c)),
            None => {
                log_err(&format!("chunk load failed, path:  {:?}", path));
                None 
            }
        }
    }).collect();

    // take all the sub-results, aggregate them into one result of same type
    reduce(sub_results)
}

fn add_or_increment<T: Copy + Eq + std::hash::Hash>(key: T, hm: &mut HashMap<T, u32>) {
    match hm.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut tx_count) => {
            tx_count.insert(tx_count.get() + 1);
        },
        std::collections::hash_map::Entry::Vacant(_) => { hm.insert(key, 1); },
    };
}

fn chunks_count<'a, T>(data: &'a Vec<T>, chunk_count: usize) -> Vec<&'a [T]> {
    let chunk_len = data.len() / chunk_count;
    let result: Vec<&[T]> = data.chunks(chunk_len).collect();
    result.to_owned()
}

fn reduce_count_map(sub_sets: Vec<PubkeyTxCountMap>) -> PubkeyTxCountMap {
    let ss_chunks: Vec<&[PubkeyTxCountMap]> = chunks_count(&sub_sets, 4);
    let sub_maps: Vec<PubkeyTxCountMap> = ss_chunks.par_iter()
    .map(|&chunk| {
        reduce_count_chunk(chunk)
    }).collect();

    //let mut outer_set = PubkeyTxCountMap::new();
    let outer_set = reduce_count_chunk(&sub_maps);

    outer_set
}

fn reduce_count_chunk(chunk: &[PubkeyTxCountMap]) -> PubkeyTxCountMap {
    let mut chunk_map = PubkeyTxCountMap::new();
    chunk.iter().for_each(|sub_map| {
        sub_map.iter().for_each(|entry|{
            let pk = *entry.0;
            let sub_count = *entry.1;
            match chunk_map.entry(pk) {
                std::collections::hash_map::Entry::Occupied(mut tx_count) => {
                    tx_count.insert(tx_count.get() + sub_count);
                },
                std::collections::hash_map::Entry::Vacant(_) => { chunk_map.insert(pk, sub_count); },
            };
        });
    });
    chunk_map
}

// given a set of .json file paths containing Solana block info,
// process them all in a streaming manner and collect the results
pub fn find_account_set_stream(block_files: &[PathBuf]) -> PubkeyTxCountMap {
    let result = time_run(|| {
        map_reduce_chunk_files::<u64, EncodedConfirmedBlock, PubkeyTxCountMap>(
            block_files,
            |p| { load_blocks_chunk_json(p) },  
            find_account_set_tuple,                         // in each chunk, count seen public keys
            reduce_count_map)                           // aggregate occurence counts into one map
    });

    let bf_len = block_files.len();
    let seconds = result.time.as_secs_f32();
    println!("total time to process {} chunks:  {:3} seconds\n", bf_len, seconds);

    result.data
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