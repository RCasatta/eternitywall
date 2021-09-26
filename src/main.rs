mod templates;
mod message;

use blocks_iterator::bitcoin::blockdata::opcodes::all::OP_RETURN;
use blocks_iterator::bitcoin::blockdata::script::Instruction;
use blocks_iterator::bitcoin::{Script, Txid};
use blocks_iterator::log::info;
use blocks_iterator::structopt::StructOpt;
use blocks_iterator::Config;
use chrono::format::StrftimeItems;
use chrono::{Datelike, NaiveDateTime, Utc};
use env_logger::Env;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, RecvError};
use templates::{create_about, create_detail_page, create_index_page, create_list_page};

#[derive(Debug)]
enum Error {
    Recv(RecvError),
}

impl From<RecvError> for Error {
    fn from(r: RecvError) -> Self {
        Error::Recv(r)
    }
}

#[derive(StructOpt, Debug, Clone)]
struct Params {
    #[structopt(flatten)]
    config: Config,

    /// Overwrite generated html files instead of skipping if they exists
    #[structopt(short, long)]
    pub overwrite: bool,
}

type MessagesByCat = BTreeMap<String, BTreeSet<message::Message>>;

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");

    let mut years_map: MessagesByCat = BTreeMap::new();
    let mut lang_map: MessagesByCat = BTreeMap::new();

    let mut params = Params::from_args();
    params.config.skip_prevout = true;
    let (send, recv) = sync_channel(100);
    let handle = blocks_iterator::iterate(params.config.clone(), send);
    while let Some(block_extra) = recv.recv()? {
        for tx in block_extra.block.txdata.iter() {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    if let Some(str) = ew_str_from_op_return(&output.script_pubkey) {
                        let txid = tx.txid();
                        let page_dirname = page_dirname(&txid);
                        let date =
                            NaiveDateTime::from_timestamp(block_extra.block.header.time as i64, 0);

                        let message = message::Message {
                            txid,
                            date,
                            msg: str.to_string(),
                        };
                        if let Some(l) = message.detect_lang() {
                            lang_map.entry(l.eng_name().to_string()).or_insert(BTreeSet::new()).insert(message.clone());
                        }

                        if !page_dirname.exists() || params.overwrite {
                            std::fs::create_dir_all(&page_dirname).unwrap();
                            let mut page_filename = page_dirname;
                            page_filename.push("index.html");
                            let page = create_detail_page(&message);
                            save_page(page_filename, page);
                        }

                        let value = years_map.entry(date.year().to_string()).or_insert(BTreeSet::new());
                        value.insert(message);
                    }
                }
            }
        }
    }
    handle.join().expect("couldn't join");
    info!("end");
    lang_map.iter().for_each(|(k,v)| {
        info!("{}: {}", k, v.len());
    });

    let mut home = PathBuf::new();
    home.push("_site");

    let index_page = create_index_page(&years_map, true);
    let mut index_file = home.clone();
    index_file.push("index.html");
    save_page(index_file, index_page);

    let lang_index_page = create_index_page(&lang_map, false);
    let mut lang_index_file = home.clone();
    lang_index_file.push("language");
    lang_index_file.push("index.html");
    save_page(lang_index_file, lang_index_page);

    for (k, v) in years_map {
        let page = create_list_page(&k.to_string(), v);
        let mut month_file = home.clone();
        month_file.push(&k.to_string());
        if !month_file.exists() {
            std::fs::create_dir_all(&month_file).unwrap();
        }
        month_file.push("index.html");
        save_page(month_file, page)
    }

    for (lang_string, v) in lang_map {
        let page = create_list_page(&lang_string, v);
        let mut lang_file = home.clone();
        lang_file.push(&lang_string);
        if !lang_file.exists() {
            std::fs::create_dir_all(&lang_file).unwrap();
        }
        lang_file.push("index.html");
        save_page(lang_file, page)
    }

    let mut about = home.clone();
    about.push("about");
    std::fs::create_dir_all(&about).unwrap();
    about.push("index.html");
    save_page(about, create_about());

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

fn now() -> String {
    let now = Utc::now().naive_utc();
    let fmt = StrftimeItems::new("%Y-%m-%d %H:%M:%S");
    format!("{}", now.format_with_items(fmt))
}

#[cfg(test)]
mod test {
    use crate::ew_str_from_op_return;
    use blocks_iterator::bitcoin::Script;
    use std::str::FromStr;

    #[test]
    fn test_parsing() {
        // op_return script in tx 0e20ae6ed9d1de7eb84823bfb4445fc3421e489c31d7694693b2fecb7d184807
        let script = Script::from_str("6a1645574275696c64696e67207468652077616c6c2e2e2e").unwrap();
        let result = ew_str_from_op_return(&script);
        assert_eq!(result, Some("Building the wall..."));
    }
}
