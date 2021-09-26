use blocks_iterator::bitcoin::Txid;
use chrono::NaiveDateTime;
use std::borrow::Cow;
use whatlang::{detect_lang, Lang};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
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

    /// https://gist.github.com/JamieMason/3748498
    /// lang set: {Fin, Ara, Rus, Ukr, Aka?, Hun, Bel, Afr, Ell, Ita, Hrv, Lat, Ron, Cat, Deu, Fra,
    /// Uzb, Lit, Sna, Ces, Aze, Kor, Tha, Srp, Bul, Jpn, Nld, Jav, Spa, Vie, Nob, Lav, Slv, Slk,
    /// Dan, Swe, Hin, Tel, Est, Eng, Cmn, Zul, Tuk, Pol, Por, Tur, Ind, Epo}
    pub fn lang(&self) -> Option<&str> {
        match self.detect_lang() {
            Some(Lang::Fin) => Some("fi"),
            Some(Lang::Ara) => Some("ar"),
            Some(Lang::Rus) => Some("ru"),
            Some(Lang::Ukr) => Some("uk"),
            Some(Lang::Hun) => Some(""),
            Some(Lang::Bel) => Some(""),
            Some(Lang::Afr) => Some(""),
            Some(Lang::Ell) => Some(""),
            Some(Lang::Ita) => Some("it"),
            Some(Lang::Hrv) => Some(""),
            Some(Lang::Lat) => Some(""),
            Some(Lang::Ron) => Some(""),
            Some(Lang::Cat) => Some(""),
            Some(Lang::Deu) => Some("de"),
            Some(Lang::Fra) => Some("fr"),
            Some(Lang::Uzb) => Some(""),
            Some(Lang::Lit) => Some(""),
            Some(Lang::Sna) => Some(""),
            Some(Lang::Ces) => Some(""),
            Some(Lang::Aze) => Some(""),
            Some(Lang::Kor) => Some(""),
            Some(Lang::Tha) => Some(""),
            Some(Lang::Srp) => Some(""),
            Some(Lang::Bul) => Some(""),
            Some(Lang::Jpn) => Some("ja"),
            Some(Lang::Nld) => Some(""),
            Some(Lang::Jav) => Some(""),
            Some(Lang::Spa) => Some("es"),
            Some(Lang::Vie) => Some(""),
            Some(Lang::Nob) => Some(""),
            Some(Lang::Lav) => Some(""),
            Some(Lang::Slv) => Some(""),
            Some(Lang::Slk) => Some(""),
            Some(Lang::Dan) => Some(""),
            Some(Lang::Swe) => Some(""),
            Some(Lang::Hin) => Some(""),
            Some(Lang::Tel) => Some(""),
            Some(Lang::Est) => Some(""),
            Some(Lang::Eng) => Some("en"),
            Some(Lang::Cmn) => Some("zh"),
            Some(Lang::Zul) => Some(""),
            Some(Lang::Tuk) => Some(""),
            Some(Lang::Pol) => Some(""),
            Some(Lang::Por) => Some("pt"),
            Some(Lang::Tur) => Some(""),
            Some(Lang::Ind) => Some(""),
            Some(Lang::Epo) => Some(""),
            _ => None,
        }
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
