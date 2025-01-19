mod message;
mod templates;

use crate::templates::create_contact;
use bitcoin_slices::bitcoin_hashes::Hash;
use bitcoin_slices::{bsl, Visit, Visitor};
use blocks_iterator::bitcoin::blockdata::opcodes::all::OP_RETURN;
use blocks_iterator::bitcoin::blockdata::script::Instruction;
use blocks_iterator::bitcoin::{Script, Txid};
use blocks_iterator::log::info;
use blocks_iterator::PipeIterator;
use chrono::format::StrftimeItems;
use chrono::{DateTime, Datelike, Utc};
use clap::Parser;
use env_logger::Env;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{fs, io};
use templates::{create_about, create_detail_page, create_index_page, create_list_page};

#[derive(Debug)]
enum Error {}

#[derive(Parser, Debug, Clone)]
struct Params {
    /// Where to produce the website
    #[clap(short, long)]
    pub target_dir: PathBuf,
}

type MessagesByCat = BTreeMap<String, BTreeSet<message::Message>>;

fn main() -> Result<(), Error> {
    init_logging();
    info!("start");

    let mut years_map: MessagesByCat = BTreeMap::new();

    let params = Params::parse();
    let mut home = params.target_dir.clone();
    home.push("site");

    let iter = PipeIterator::new(io::stdin(), None);

    for block_extra in iter {
        let msgs = find_msg_in_block(block_extra.block_bytes());
        for message in msgs {
            let page_dirname = page_dirname(&home, &message.txid);

            let mut page_filename = page_dirname;
            page_filename.push("index.html");
            let page = create_detail_page(&message);
            save_page(page_filename, page);

            let value = years_map
                .entry(message.date.year().to_string())
                .or_insert(BTreeSet::new());
            value.insert(message);
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

fn find_msg_in_block(block_bytes: &[u8]) -> Vec<message::Message> {
    let mut visitor = BlockVisitor::new();

    bsl::Block::visit(block_bytes, &mut visitor).expect("visit fail");

    let mut messages = vec![];
    for (txid, str) in visitor.messages.iter() {
        let date = DateTime::from_timestamp(visitor.time as i64, 0).expect("invalid timestamp");

        let message = message::Message {
            txid: *txid,
            date,
            msg: str.to_string(),
        };
        messages.push(message);
    }
    messages
}

struct BlockVisitor {
    messages: Vec<(Txid, String)>,
    zero_txid: Txid,
    time: u32,
}
impl BlockVisitor {
    fn new() -> Self {
        Self {
            messages: vec![],
            zero_txid: Hash::all_zeros(),
            time: 0,
        }
    }
}
impl Visitor for BlockVisitor {
    fn visit_block_header(&mut self, header: &bsl::BlockHeader) -> core::ops::ControlFlow<()> {
        self.time = header.time();
        core::ops::ControlFlow::Continue(())
    }
    fn visit_tx_out(&mut self, _vout: usize, tx_out: &bsl::TxOut) -> core::ops::ControlFlow<()> {
        let script = tx_out.as_bitcoin_script();
        if script.is_op_return() {
            if let Some(s) = ew_str_from_op_return(&script) {
                self.messages.push((self.zero_txid, s));
            }
        }

        core::ops::ControlFlow::Continue(())
    }
    fn visit_transaction(&mut self, tx: &bsl::Transaction) -> core::ops::ControlFlow<()> {
        for (txid, _) in self.messages.iter_mut() {
            if self.zero_txid == *txid {
                *txid = tx.txid().into();
            }
        }
        core::ops::ControlFlow::Continue(())
    }
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

fn ew_str_from_op_return(script: &Script) -> Option<String> {
    let mut instructions = script.instructions();
    if let Some(Ok(Instruction::Op(all))) = instructions.next() {
        if all == OP_RETURN {
            if let Some(Ok(Instruction::PushBytes(push_bytes))) = instructions.next() {
                let bytes = push_bytes.as_bytes();
                if bytes.len() > 2 && bytes[0] == 0x45 && bytes[1] == 0x57 {
                    return Some(std::str::from_utf8(&bytes[2..]).ok()?.to_string());
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

fn init_logging() {
    let mut builder = env_logger::Builder::from_env(Env::default().default_filter_or("info"));
    if let Ok(s) = std::env::var("RUST_LOG_STYLE") {
        if s == "SYSTEMD" {
            builder.format(|buf, record| {
                let level = match record.level() {
                    log::Level::Error => 3,
                    log::Level::Warn => 4,
                    log::Level::Info => 6,
                    log::Level::Debug => 7,
                    log::Level::Trace => 7,
                };
                writeln!(buf, "<{}>{}: {}", level, record.target(), record.args())
            });
        }
    }

    builder.init();
}

#[cfg(test)]
mod test {
    use crate::ew_str_from_op_return;
    use blocks_iterator::bitcoin::Script;

    #[test]
    fn test_parsing() {
        // op_return script in tx 0e20ae6ed9d1de7eb84823bfb4445fc3421e489c31d7694693b2fecb7d184807
        let bytes = [
            0x6au8, 0x16, 0x45, 0x57, 0x42, 0x75, 0x69, 0x6c, 0x64, 0x69, 0x6e, 0x67, 0x20, 0x74,
            0x68, 0x65, 0x20, 0x77, 0x61, 0x6c, 0x6c, 0x2e, 0x2e, 0x2e,
        ];
        let script = Script::from_bytes(&bytes[..]);
        let result = ew_str_from_op_return(&script);
        assert_eq!(result, Some("Building the wall...".to_string()));
    }

    #[test]
    fn test_find_msg_in_block() {
        let block_bytes = include_bytes!(
            "../test_data/000000000000000001536d790f5792bc015136dfee015ead92116beb32db878b.bin"
        );
        let msgs = crate::find_msg_in_block(&block_bytes[..]);
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].msg, " BlockCypher Team = Awesome");
        assert_eq!(msgs[0].date.to_string(), "2015-06-28 00:21:37 UTC");
        assert_eq!(
            msgs[0].txid.to_string(),
            "8593956e0eef311a58b324b682b5edbd2ac43e2ef5829d07fc0000cb89a22516"
        );
    }
}
