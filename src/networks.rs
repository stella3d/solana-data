use crate::util::dbg_println_each_indent;


// for convenience, several keywords expand to default RPC node URLs on each Solana network
pub(crate) const DEVNET_WORD: &str = "dev";
pub(crate) const DEVNET_RPC: &str = "https://api.devnet.solana.com";
pub(crate) const TESTNET_WORD: &str = "test";
pub(crate) const TESTNET_RPC: &str = "https://api.testnet.solana.com";
pub(crate) const MAINNET_WORD: &str = "main";
pub(crate) const MAINNET_RPC: &str = "https://api.mainnet-beta.solana.com";

pub(crate) const DEFAULT_NET_RPCS: [&str; 3] = [ DEVNET_RPC, TESTNET_RPC, MAINNET_RPC ];

pub(crate) fn expand_rpc_keywords(rpc_input: &str) -> &str {
    match rpc_input.to_lowercase().as_str() {  
        DEVNET_WORD => DEVNET_RPC,
        TESTNET_WORD => TESTNET_RPC,
        MAINNET_WORD => MAINNET_RPC,
        lc_input => {
            // show this help if a non-url or default rpc url is given
            if !lc_input.starts_with("https://") || DEFAULT_NET_RPCS.contains(&lc_input) { 
                println!("\nthe following keywords are aliases for the default public RPC node on each network:");
                dbg_println_each_indent(&DEFAULT_NET_RPCS, true);
                println!("these keywords can be used as the --rpc argument, in place of a URL\n");
            }
            rpc_input
        }
    }
}