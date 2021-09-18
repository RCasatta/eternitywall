use blocks_iterator::{periodic_log_level, Config};
use env_logger::Env;
use log::{info, log, warn};
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::sync_channel;
use structopt::StructOpt;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");

    let mut map = HashMap::new();

    let mut config = Config::from_args();
    config.skip_prevout = true;
    let (send, recv) = sync_channel(100);
    let handle = blocks_iterator::iterate(config, send);
    while let Some(block_extra) = recv.recv()? {
        for tx in block_extra.block.txdata.iter() {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    if let Ok(str) = std::str::from_utf8(&output.script_pubkey.as_bytes()[1..]) {
                        map.insert(tx.txid(), (block_extra.block.header.time, str.to_string()));
                    }
                }
            }
        }
        log!(
            periodic_log_level(block_extra.height),
            "map size: {}",
            map.len()
        )
    }
    handle.join().expect("couldn't join");
    info!("end");
    Ok(())
}
