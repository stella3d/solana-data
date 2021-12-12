# _solana-data_
toy CLI tool for accessing and analyzing Solana blockchain data

# **Usage**

## **Building**

in the project, run: 

`cargo build --release`

Help text can be accessed at the command line

## **Tasks**
All top-level functionality is in a named ___task___.

The task argument is `--task` / `-t`, used like:

```
sol-data -t count_txs
```

where `count_txs` is the task name.

>on Windows, use `sol-data.exe` instead of `sol-data`
#  
### Supported Tasks

Supply one of these names exactly to `--task` / `-t`

* ### **scrape_blocks**
    Must run this (or restore from archive) before data is available for other tasks.

    Repeatedly fetch recent block data from a Solana RPC node, and save as .json files in '_blocks/json_'

    Accepts an argument `--minutes` / `-m` specifying how long to scrape blocks for, in minutes.

    This would run for 2 hours:
    
    ```
    sol-data -t scrape_blocks -m 120
    ```
* ### **chunk_blocks**

    Take a batch of single-block files and group them into larger files.
    
    `sol-data -t chunk_blocks`
* ### **count_txs**

    Count how many times each public key is involved in a transaction, within the given blocks.

    `sol-data -t count_txs`
* ### **mean_fsize**

    Calculate the average size of downloaded Solana block .json files in '_blocks/json_'.
    
    `sol-data -t mean_fsize`
* ### **cmp_block_loads**

    Run performance test for effect of file size on loading & processing many chunks of saved blocks.

    `sol-data -t cmp_block_loads`
* ### **block_sample**

    Copy a small sample of the base 'blocks/json' data set to 'blocks/json_sample'.

    `sol-data -t block_sample`