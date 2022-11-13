use blocks_iterator::bitcoin::Txid;
use chrono::NaiveDateTime;
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
    use blocks_iterator::bitcoin::Txid;
    use chrono::NaiveDateTime;
    use whatlang::detect;

    #[test]
    fn test_detect() {
        let detected =
            detect("Non lungo che tocchi, non largo che otturi, ma duro che duri.").unwrap();
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
