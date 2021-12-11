use std::{cmp::max, time::Duration, fs};
use serde::{Serialize, Deserialize};

use crate::{util::{log_err, loop_task}, client::get_client, files};


#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub(crate) struct ScrapeState {
    pub last_slot: u64,
}

impl Default for ScrapeState {
    fn default() -> Self {
        Self { last_slot: Default::default() }
    }
}

pub(crate) fn scrape_blocks(previous_state: ScrapeState, rpc_url: &str) -> Option<ScrapeState> {
    println!("using rpc url:  {}\n", rpc_url);
    let mut client = get_client(rpc_url);

    let slot_res= client.rpc.get_slot();
    let slot = match slot_res {
        Ok(s) => s,
        Err(e) => { 
            eprintln!("{}", e);
            0u64 
        }
    };
    if slot <= 0 {
        return None; 
    }

    let backlog_limit: usize = 1024;
    let slot_count = max(slot - previous_state.last_slot, backlog_limit as u64);
    let slots_r = client.rpc.get_blocks_with_limit(slot_count, backlog_limit);
    let slots = match slots_r {
        Ok(s) => s,
        Err(_) => vec![],
    };

    println!("\n{} slots to request: {}-{}\n", slots.len(), slots.first().unwrap(), slots.last().unwrap());

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

pub(crate) fn do_scrape(rpc_url: &str) {
    let prev_state = load_state();
    println!("\nDO LOOP");
    println!("\nloaded previous run's state from file:\n{:?}", prev_state);

    match prev_state {
        Ok(s) => {
            if let Some(new_state) = scrape_blocks(s, rpc_url) {
                save_state(new_state); 
            };
        },
        Err(e) => log_err(&e)
    }
}

pub(crate) fn scrape_loop(duration: Duration, rpc_url: &str) {
    let task = || { do_scrape(rpc_url) };
    loop_task(duration, task);
}

const STATE_FILE: &str = "scrape_state.json";

fn save_state(state: ScrapeState) {
    fs::write(STATE_FILE, serde_json::to_vec(&state).unwrap()).unwrap();
}

fn load_state() -> Result<ScrapeState, serde_json::Error> {
    match fs::read(STATE_FILE) {
        Ok(data) => serde_json::from_str::<ScrapeState>
                            (std::str::from_utf8(&data).unwrap()),
        Err(_) => Ok(ScrapeState { last_slot: 0 }),
    }
}