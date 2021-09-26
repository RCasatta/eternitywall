use blocks_iterator::bitcoin::Txid;
use chrono::NaiveDateTime;
use std::borrow::Cow;
use whatlang::{detect_lang, Lang};
use std::cmp::Ordering;
use isolang::Language;

pub struct Message {
    pub txid: Txid,
    pub date: NaiveDateTime,
    pub msg: String,
}

impl Message {

    pub fn escape_msg(&self) -> Cow<str> {
        html_escape::encode_text(&self.msg)
    }

    pub fn link(&self) -> String {
        format!("/m/{}", self.txid)
    }

    pub fn lang(&self) -> &str {
        if let Some(l) = self.detect_lang() {
            if let Some(l) = Language::from_639_3(l.code()) {
                return l.to_639_1().unwrap_or("en");
            }
        }
        "en"
    }

    pub fn detect_lang(&self) -> Option<Lang> {
        detect_lang(&self.msg)
    }

    pub fn date(&self) -> String {
        self.date.to_string()
    }
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
