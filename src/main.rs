mod message;
mod templates;

use crate::templates::create_contact;
use blocks_iterator::bitcoin::blockdata::opcodes::all::OP_RETURN;
use blocks_iterator::bitcoin::blockdata::script::Instruction;
use blocks_iterator::bitcoin::{Script, Txid};
use blocks_iterator::log::info;
use blocks_iterator::structopt::StructOpt;
use blocks_iterator::PipeIterator;
use chrono::format::StrftimeItems;
use chrono::{DateTime, Datelike, Utc};
use env_logger::Env;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};
use templates::{create_about, create_detail_page, create_index_page, create_list_page};

#[derive(Debug)]
enum Error {}

#[derive(StructOpt, Debug, Clone)]
struct Params {
    /// Where to produce the website
    #[structopt(short, long)]
    pub target_dir: PathBuf,
}

type MessagesByCat = BTreeMap<String, BTreeSet<message::Message>>;

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");

    let mut years_map: MessagesByCat = BTreeMap::new();

    let params = Params::from_args();
    let mut home = params.target_dir.clone();
    home.push("site");

    let iter = PipeIterator::new(io::stdin(), None);

    for block_extra in iter {
        for (txid, tx) in block_extra.iter_tx() {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    if let Some(str) = ew_str_from_op_return(&output.script_pubkey) {
                        let page_dirname = page_dirname(&home, &txid);
                        let date =
                            DateTime::from_timestamp(block_extra.block.header.time as i64, 0)
                                .expect("invalid timestamp");

                        let message = message::Message {
                            txid: *txid,
                            date,
                            msg: str.to_string(),
                        };

                        let mut page_filename = page_dirname;
                        page_filename.push("index.html");
                        let page = create_detail_page(&message);
                        save_page(page_filename, page);

                        let value = years_map
                            .entry(date.year().to_string())
                            .or_insert(BTreeSet::new());
                        value.insert(message);
                    }
                }
            }
        }
    }
    info!("end");

    let index_page = create_index_page(&years_map, true);
    let mut index_file = home.clone();
    index_file.push("index.html");
    save_page(index_file, index_page);

    for (k, v) in years_map {
        let page = create_list_page(&k.to_string(), v);
        let mut month_file = home.clone();
        month_file.push(&k.to_string());
        month_file.push("index.html");
        save_page(month_file, page)
    }

    let mut about = home.clone();
    about.push("about");
    about.push("index.html");
    save_page(about, create_about());

    let mut contact = home.clone();
    contact.push("contact");
    contact.push("index.html");
    save_page(contact, create_contact());

    // favicon
    let mut favicon_file = home.clone();
    favicon_file.push("favicon.ico");
    let favicon = include_bytes!("../target_dir/site/favicon.ico");
    fs::write(favicon_file, &favicon).unwrap();

    Ok(())
}

fn page_dirname(home: &PathBuf, txid: &Txid) -> PathBuf {
    let mut path = home.clone();
    path.push("m");
    path.push(txid.to_string());
    path
}

fn save_page(filename: PathBuf, page: String) {
    let parent = filename.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent).unwrap();
    }
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
