# _solana-data_
toy CLI tool for accessing and analyzing Solana blockchain data

# **Building**

in the project, run: 

```
cargo build --release
```

The executable should be output to _**`target/release/sol-data(.exe)`**_

>on Windows, use `sol-data.exe` wherever the docs say `sol-data`


# **Usage**



## **Help**

Help text can be printed with `--help` / `-h`
```
sol-data --help
```

## **Tasks**
All top-level functionality is in a named ___task___.

The task argument is `--task` / `-t`

```
sol-data -t <task-name>
```

### _Supported Tasks_

Use one of these names as the `--task` / `-t` arg to run it.

* ### **scrape_blocks**
    Repeatedly fetch detailed recent block data from a Solana RPC node.
    
    Saves blocks as files, in _`blocks/json/slot_*.json`_

    >Must run this before data is available for other tasks.

    Accepts 2 arguments:
    * `--minutes` / `-m`
        
        How long to scrape blocks for, in minutes.

        This would run for an hour:
        ```
        sol-data -t scrape_blocks -m 60
        ```
    * `--rpc` / `-r`
        
        URL of the RPC node to use for requests.
        
        ###### Examples
        * fetch from mainnet for 2 hours.
            ```
            sol-data -t scrape_blocks -m 120 --rpc https://api.mainnet-beta.solana.com
            ```
        * fetch from testnet for 30 minutes
            ```
            sol-data -t scrape_blocks -m 30 --rpc https://api.testnet.solana.com
            ```

        Convenience labels for the the default public RPC nodes are built in.

        * '_dev_' for devnet,
        
        * '_test_' for testnet,

        * '_main_' for mainnet-beta


        using
        ```
        --rpc main
        ```
        is the same as
        ```
        --rpc https://api.mainnet-beta.solana.com
        ```

* ### **chunk_blocks**
    Take a directory of many single-block _.json_ files, and group them into larger 'chunk' files.

    Outputs new files to _`blocks/json_chunked/slots_*.json`_ 

    >Requires _.json_ files from **`scrape_blocks`** task to be in _`blocks/json/`_

    Accepts 1 argument:
    * `--chunk_mb`
        
        The max size of the input data (in megabytes) grouped into 1 output chunk / file.


    ###### Example
    Create ~2MB, then ~8MB chunk files.
    ```
    sol-data -t chunk_blocks --chunk_mb 2;
    sol-data -t chunk_blocks --chunk_mb 8;
    ```
* ### **count_txs**
    Count how many times each public key is seen in the given blocks' transactions.
    
    Outputs the key counts to a file, _`blocks/key_tx_count_*.json`_

    >Requires _.json_ files output from the **`chunk_blocks`** task to be in _`blocks/json_chunked`_
    ###### Example
    ```
    sol-data -t count_txs
    ```
* ### **mean_fsize**
    Calculate the average size of downloaded Solana blocks' .json files.
    
    >Requires _.json_ files from **`scrape_blocks`** task to be in _`blocks/json/`_
    ###### Example
    ```
    sol-data -t mean_fsize
    ```
* ### **cmp_block_loads**
    Run performance test, looking at the effect of file size on loading & processing many chunks of blocks.
    
    >Requires folders of differently-chunked _.json_ files, from **`chunk_blocks`** task runs, to be in _`blocks/json/sized`_

    ###### Example
    ```
    sol-data -t cmp_block_loads
    ```
* ### **block_sample**
    Copy a sample (1/50 files) of _`blocks/json/*.json`_ data to _`blocks/json_sample`_.

    >Requires _.json_ files from **`scrape_blocks`** task to be in _`blocks/json/`_

    ###### Example
    ```
    sol-data -t block_sample
    ```
#  
