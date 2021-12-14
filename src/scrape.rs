use std::{time::Duration, fs, cmp::max};
use serde::{Serialize, Deserialize};

use crate::{util::{log_err, loop_task, minutes_duration}, client::SolClient, files, cli::CliArguments, scrape};


pub(crate) fn do_scrape(rpc_url: &str) {
    match load_state() {
        Ok(s) => {
            println!("\nloaded previous run's state from file:\n{:?}", s);
            if let Some(new_state) = scrape_blocks(s, rpc_url) {
                save_state(new_state); 
            };
        },
        Err(e) => {
            log_err(&e);
            // we can still run ok, so use default
            scrape_blocks(ScrapeState { last_slot: 0 }, rpc_url);
        }
    }
}

// request data of recent blocks from an RPC node, and save them to disk
fn scrape_blocks(previous_state: ScrapeState, rpc_url: &str) -> Option<ScrapeState> {
    println!("using rpc url:  {}\n", rpc_url);
    let mut client = SolClient::get(rpc_url);

    let slot_res= client.rpc.get_slot();
    let slot = match slot_res {
        Ok(s) => s,
        Err(e) => { log_err(&e); return None }
    };

    let start = max(previous_state.last_slot, slot - 512);
    let slots = match client.rpc.get_blocks_with_limit(start, 1024) {
        Ok(s) => s,
        Err(e) => { log_err(&e); return None },
    };

    println!("\nslots to request:  {}\n", slots.len());

    let last = client.get_block_details(&slots,
        |(slot, ecb)| {
            match *ecb {
                Some(b) => {
                    //println!("write range block file:  slot_{}", slot);
                    files::write_json_encoded_block(*slot, b);
                },
                None => {},
            }
        });

    if last == 0 { None }
    else { Some(ScrapeState { last_slot: last }) }
}

pub(crate) fn scrape_loop(duration: Duration, rpc_url: &str) {
    let task = || { do_scrape(rpc_url) };
    loop_task(duration, task);
}

pub(crate) fn scrape_with_args(cli_args: &CliArguments) {
    let mins = cli_args.minutes.unwrap_or(60);
    let duration = minutes_duration(mins);

    match cli_args.rpc.as_ref() {
        Some(rpc) => {
            println!("\nscraping blocks for {} minutes, from RPC node:  {}\n", mins, rpc);
            scrape::scrape_loop(duration, &rpc);
        },
        None => {
            eprintln!("\nSolana RPC url required, but not provided\n");
        },
    };
}


#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub(crate) struct ScrapeState {
    pub last_slot: u64,
}

impl Default for ScrapeState {
    fn default() -> Self {
        Self { last_slot: Default::default() }
    }
}

const STATE_FILE: &str = "scrape_state.json";
// lack of error handling here isn't ideal, but it's also not that important
fn save_state(state: ScrapeState) {
    match serde_json::to_vec(&state) {
        Ok(bytes) => { 
            if let Err(e) = fs::write(STATE_FILE, bytes) { 
                log_err(&e); 
            } 
        },
        Err(e) => { log_err(&e) }
    };
}

fn load_state() -> Result<ScrapeState, serde_json::Error> {
    match fs::read(STATE_FILE) {
        Ok(data) => {
            serde_json::from_slice::<ScrapeState>(&data)
        },
        Err(_) => Ok(ScrapeState { last_slot: 0 }),
    }
}
