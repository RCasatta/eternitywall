use bitcoin::blockdata::opcodes::all::OP_RETURN;
use bitcoin::blockdata::script::Instruction;
use bitcoin::{Script, Txid};
use blocks_iterator::Config;
use chrono::{Datelike, NaiveDateTime, Utc};
use env_logger::Env;
use log::info;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{sync_channel, RecvError};
use structopt::StructOpt;

#[derive(Debug)]
enum Error {
    Recv(RecvError),
}

impl From<RecvError> for Error {
    fn from(r: RecvError) -> Self {
        Error::Recv(r)
    }
}

struct Message {
    txid: Txid,
    date: NaiveDateTime,
    msg: String,
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> Ordering {
        match other.date.cmp(&self.date) {
            Ordering::Equal => self.txid.cmp(&other.txid),
            ord => ord,
        }
    }
}
impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}
impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.txid == other.txid && self.date == other.date
    }
}
impl Eq for Message {}

type MessagesByMonth = BTreeMap<i32, BTreeSet<Message>>;

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");

    let mut map: MessagesByMonth = BTreeMap::new();

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
                        let date =
                            NaiveDateTime::from_timestamp(block_extra.block.header.time as i64, 0);

                        let message = Message {
                            txid,
                            date,
                            msg: str.to_string(),
                        };

                        if !page_dirname.exists() {
                            std::fs::create_dir_all(&page_dirname).unwrap();
                            let mut page_filename = page_dirname;
                            page_filename.push("index.html");
                            let page = create_detail_page(&message);
                            save_page(page_filename, page);
                        }

                        let value = map.entry(date.year()).or_insert(BTreeSet::new());
                        value.insert(message);
                    }
                }
            }
        }
    }
    handle.join().expect("couldn't join");
    info!("end");

    let index_page = create_index_page(&map);
    let mut index_file = PathBuf::new();
    index_file.push("_site");
    index_file.push("index.html");
    save_page(index_file, index_page);

    let mut home = PathBuf::new();
    home.push("_site");
    for (k, v) in map {
        let page = create_year_page(k, v);
        let mut month_file = home.clone();
        month_file.push(&k.to_string());
        if !month_file.exists() {
            std::fs::create_dir_all(&month_file).unwrap();
        }
        month_file.push("index.html");
        save_page(month_file, page)
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

fn create_index_page(map: &MessagesByMonth) -> String {
    let mut list = String::new();
    for (year, messages) in map {
        list.push_str("<li><a href=\"/");
        list.push_str(&year.to_string());
        list.push_str("\">");
        list.push_str(&year.to_string());
        list.push_str(" (");
        list.push_str(&messages.len().to_string());
        list.push_str(")");
        list.push_str("</a></li>");
    }
    let now = Utc::now().naive_utc();

    format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><h1>EternityWall</h1><ul>{}</ul><p><a href=\"/about\">About</a></p><p>Created {}</p></body></html>", list, now)
}

fn create_about() -> String {
    "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><h1><a href=\"/\">Eternity Wall</a></h1><p>EternityWall shows message in the Bitcoin blockchain. A message is a transaction with an OP_RETURN output containing valid utf-8 starting with characters \"EW\". All dates are referred to the block timestamp containing the transaction and are in UTC.</p></body></html>".to_string()
}

fn create_year_page(year: i32, messages: BTreeSet<Message>) -> String {
    let mut list = String::new();

    for msg in messages {
        let txid = format!("{}", msg.txid);
        list.push_str("<li><a href=\"/m/");
        list.push_str(&txid);
        list.push_str("\">");
        list.push_str(&msg.date.to_string());
        list.push_str("</a> - ");
        list.push_str(&msg.msg);
        list.push_str("</li>");
    }
    format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><h1><a href=\"/\">Eternity Wall</a></h1><h2>{}</h2><ul>{}</ul></body></html>", year, list)
}

fn create_detail_page(msg: &Message) -> String {
    format!("<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><h1><a href=\"/\">Eternity Wall</a></h1><p>{} UTC</p><h1>{}</h1></body></html>", msg.date, msg.msg)
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
    use crate::{create_detail_page, create_index_page, ew_str_from_op_return, MessagesByMonth, Message};
    use bitcoin::{Script, Txid};
    use chrono::NaiveDateTime;
    use std::str::FromStr;
    use std::collections::BTreeSet;

    #[test]
    fn test_parsing() {
        // op_return script in tx 0e20ae6ed9d1de7eb84823bfb4445fc3421e489c31d7694693b2fecb7d184807
        let script = Script::from_str("6a1645574275696c64696e67207468652077616c6c2e2e2e").unwrap();
        let result = ew_str_from_op_return(&script);
        assert_eq!(result, Some("Building the wall..."));
    }

    #[test]
    fn test_page_detail() {
        let date = NaiveDateTime::from_timestamp(1445192722 as i64, 0);
        let msg = Message {
            msg: "Atoms are made of universes".to_string(),
            date,
            txid: Txid::default(),
        };
        let result = create_detail_page(&msg);
        assert_eq!(result, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><p>2015-10-18 18:25:22 UTC</p><h1>Atoms are made of universes</h1></body></html>");
    }

    #[test]
    fn test_page_index() {
        let mut map = MessagesByMonth::new();
        map.insert("2019-01".to_string(), BTreeSet::new());
        map.insert("2019-02".to_string(), BTreeSet::new());

        let result = create_index_page(&map);
        assert_eq!(result, "<!DOCTYPE html><html><head><meta charset=\"utf-8\"/></head><body><h1>EternityWall</h1><ul><li><a href=\"/2019-01\">2019-01 (0)</a></li><li><a href=\"/2019-02\">2019-02 (0)</a></li></ul></body></html>");
    }
}
