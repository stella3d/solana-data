# solana-data
toy CLI tool for accessing and analyzing Solana blockchain data

## Usage

All top-level functionality is in a named _task_.

The existing tasks are:

* "**scrape_blocks**"
    
    Must be run before data is available for others.

    Repeatedly fetch & save recent block data in JSON form from a Solana RPC node.
* "**chunk_blocks**"

    Take a batch of single-block files and group them into larger files.
* "**count_txs**"

    Count how many times each public key is involved in a transaction, within the given blocks
* "**mean_fsize**"

    Calculate the average size of downloaded Solana block .json files in 'blocks/json'
* "**cmp_block_loads**"

    Run performance test for effect of file size on loading & processing many chunks of saved blocks
* "**block_sample**"

    Copy a small sample of the base 'blocks/json' data set to 'blocks/json_sample'

