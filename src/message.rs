use blocks_iterator::bitcoin::Txid;
use chrono::NaiveDateTime;
use whatlang::{detect, Lang};
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct Message {
    pub txid: Txid,
    pub date: NaiveDateTime,
    pub msg: String,
}

impl Message {

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
            Some(Lang::Hun) => Some("hu"),
            Some(Lang::Bel) => Some("be"),
            Some(Lang::Afr) => Some("af"),
            Some(Lang::Ell) => Some("el"),
            Some(Lang::Ita) => Some("it"),
            Some(Lang::Hrv) => Some("hr"),
            Some(Lang::Ron) => Some("ro"),
            Some(Lang::Cat) => Some("ca"),
            Some(Lang::Deu) => Some("de"),
            Some(Lang::Fra) => Some("fr"),
            Some(Lang::Uzb) => Some("uz"),
            Some(Lang::Lit) => Some("lt"),
            Some(Lang::Ces) => Some("cs"),
            Some(Lang::Aze) => Some("az"),
            Some(Lang::Kor) => Some("ko"),
            Some(Lang::Tha) => Some("th"),
            Some(Lang::Srp) => Some("sr"),
            Some(Lang::Bul) => Some("bg"),
            Some(Lang::Jpn) => Some("ja"),
            Some(Lang::Nld) => Some("nl"),
            Some(Lang::Spa) => Some("es"),
            Some(Lang::Vie) => Some("vi"),
            Some(Lang::Nob) => Some("nb-NO"),
            Some(Lang::Lav) => Some("lv"),
            Some(Lang::Slv) => Some("sl"),
            Some(Lang::Slk) => Some("sk"),
            Some(Lang::Dan) => Some("da"),
            Some(Lang::Swe) => Some("sv"),
            Some(Lang::Hin) => Some("hi"),
            Some(Lang::Tel) => Some("te"),
            Some(Lang::Est) => Some("et"),
            Some(Lang::Eng) => Some("en"),
            Some(Lang::Cmn) => Some("zh"),
            Some(Lang::Zul) => Some("zu-ZA"),
            Some(Lang::Pol) => Some("pl"),
            Some(Lang::Por) => Some("pt"),
            Some(Lang::Tur) => Some("tr"),
            Some(Lang::Ind) => Some("id"),
            Some(Lang::Sna) => None,
            Some(Lang::Jav) => None,
            Some(Lang::Tuk) => None,
            Some(Lang::Lat) => None,
            Some(Lang::Epo) => None,
            _ => None,
        }
    }

    pub fn detect_lang(&self) -> Option<Lang> {
        detect(&self.msg).filter(|i| i.confidence() > 0.3 ).map(|i| i.lang())
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


#[cfg(test)]
pub mod test {
    use super::Message;
    use chrono::NaiveDateTime;
    use blocks_iterator::bitcoin::Txid;
    use whatlang::{detect};


    #[test]
    fn test_detect() {

        let detected = detect("Non lungo che tocchi, non largo che otturi, ma duro che duri.").unwrap();
        println!("{:?}", detected);

    }

    pub fn get_message() -> Message {
        Message {
            msg: "Atoms are made of universes".to_string(),
            date: NaiveDateTime::from_timestamp(1445192722 as i64, 0),
            txid: Txid::default(),
        }
    }
    pub fn get_another_message() -> Message {
        Message {
            msg: "Ciao mi chiamo Gianni e sono italiano".to_string(),
            date: NaiveDateTime::from_timestamp(1445194722 as i64, 0),
            txid: Txid::default(),
        }
    }
}