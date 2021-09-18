use bitcoin::blockdata::opcodes::all::OP_RETURN;
use bitcoin::blockdata::script::Instruction;
use bitcoin::{Script, Txid};
use blocks_iterator::Config;
use env_logger::Env;
use log::info;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, RecvError};
use structopt::StructOpt;
use chrono::NaiveDateTime;

#[derive(Debug)]
enum Error {
    Recv(RecvError),
}

impl From<RecvError> for Error {
    fn from(r: RecvError) -> Self {
        Error::Recv(r)
    }
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");
    let mut config = Config::from_args();
    config.skip_prevout = true;
    let (send, recv) = sync_channel(100);
    let handle = blocks_iterator::iterate(config, send);
    while let Some(block_extra) = recv.recv()? {
        for tx in block_extra.block.txdata.iter() {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    if let Some(str) = ew_str_from_op_return(&output.script_pubkey) {
                        let txid = tx.txid();
                        let page_dirname = page_dirname(&txid);
                        if !page_dirname.exists() {
                            std::fs::create_dir_all(&page_dirname).unwrap();
                            let mut page_filename = page_dirname;
                            page_filename.push("index.html");
                            let page = create_page( block_extra.block.header.time, str);
                            save_page(page_filename, page);
                        }
                    }
                }
            }
        }
    }
    handle.join().expect("couldn't join");
    info!("end");
    Ok(())
}

fn page_dirname(txid: &Txid) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("_site");
    path.push("m");
    path.push(txid.to_string());
    path
}

fn save_page(filename: PathBuf, page: String) {
    let mut file = File::create(filename).unwrap();
    file.write(page.as_bytes()).unwrap();
}

fn create_page(time: u32, msg: &str) -> String {
    let date = NaiveDateTime::from_timestamp(time as i64, 0);
    format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><p>{} UTC</p><h1>{}</h1></body></html>", date, msg)
}

fn ew_str_from_op_return(script: &Script) -> Option<&str> {
    let mut instructions = script.instructions();
    if let Some(Ok(Instruction::Op(all))) = instructions.next() {
        if all == OP_RETURN {
            if let Some(Ok(Instruction::PushBytes(bytes))) = instructions.next() {
                if bytes.len() > 2 && bytes[0] == 0x45 && bytes[1] == 0x57 {
                    return Some(std::str::from_utf8(&bytes[2..]).ok()?);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod test {
    use bitcoin::consensus::deserialize;
    use bitcoin::hashes::hex::FromHex;
    use bitcoin::Script;
    use std::str::FromStr;
    use crate::{ew_str_from_op_return, create_page};

    #[test]
    fn test_parsing() {
        // op_return script in tx 0e20ae6ed9d1de7eb84823bfb4445fc3421e489c31d7694693b2fecb7d184807
        let script = Script::from_str("6a1645574275696c64696e67207468652077616c6c2e2e2e").unwrap();
        let result = ew_str_from_op_return(&script);
        assert_eq!(result, Some("Building the wall..."));
    }

    #[test]
    fn test_page() {
        let result = create_page(1445192722, "Atoms are made of universes");
        assert_eq!(result, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><p>2015-10-18 18:25:22 UTC</p><h1>Atoms are made of universes</h1></body></html>");
    }
}
