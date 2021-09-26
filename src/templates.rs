use crate::{now, MessagesByCat};
use crate::message::Message;
use maud::{html, Markup, DOCTYPE};
use std::collections::BTreeSet;

/// A basic header with a dynamic `page_title`.
fn header() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";

            title { "EternityWall" }
        }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        footer {
            a href="/about" { "About" }
            p { "Page created " (now()) }
        }

    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            (header())
            body {
                h1 { a href="/" { "EternityWall" } }
                (content)
                (footer())
            }
        }
    }
}

pub fn create_index_page(map: &MessagesByCat, reverse: bool) -> String {
    let mut cats: Vec<_> = map.keys().collect();
    if reverse {
        cats.reverse();
    }
    let list = html! {
        ul {
            @for cat in cats {
                li {
                    a href=(link_cat(cat)) { (cat) }
                    " ("
                    (map.get(cat).unwrap().len().to_string())
                    ")"
                }
            }
        }
    };

    page(list).into_string()
}

pub fn create_about() -> String {
    let link = html! {
        a href="https://blog.eternitywall.com/2016/06/01/how-to-write-a-message-on-the-blockchain" { "write a message" }
    };
    let content = html! {
        p { "EternityWall shows message in the Bitcoin blockchain." }
        p { "A message is a transaction with an OP_RETURN output containing valid utf-8 starting with characters \"EW\"." }
        p { "All dates are referred to the block timestamp containing the transaction and are in UTC." }
        p { "How to " (link) " with Bitcoin Core" }
    };

    page(content).into_string()
}

pub fn create_list_page(title: &str, messages: BTreeSet<Message>) -> String {
    let list = html! {
        h2 { (title) }
        ul {
            @for msg in &messages {
                @if let Some(lang) = msg.lang() {
                    li {
                        a href=(msg.link()) { (msg.date()) }
                        " - "
                        span lang=(lang) { (msg.escape_msg()) }
                    }
                } @else {
                    li {
                        a href=(msg.link()) { (msg.date()) }
                        " - "
                        { (msg.escape_msg()) }
                    }
                }

            }
        }
    };

    page(list).into_string()
}

pub fn create_detail_page(msg: &Message) -> String {
    let content = html! {
        h2 { (msg.date()) " UTC" }
        @if let Some(lang) = msg.lang() {
            h1 { span lang=(lang) { (msg.escape_msg()) }  }
        } @else {
            h1 { (msg.escape_msg()) }
        }

    };

    page(content).into_string()
}

fn link_cat(cat: &str) -> String {
    format!("/{}", cat)
}

#[cfg(test)]
mod test {
    use crate::templates::{create_detail_page, create_index_page, create_list_page, page};
    use crate::{MessagesByMonth};
    use crate::message::Message;
    use blocks_iterator::bitcoin::Txid;
    use chrono::NaiveDateTime;
    use maud::html;
    use std::collections::BTreeSet;
    use whatlang::detect_lang;

    #[test]
    fn test_page() {
        let content = html! { p { "Hello" } };
        let page = page(content).into_string();
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_page_detail() {
        let msg = get_message();
        let page = create_detail_page(&msg);
        assert_eq!("", page);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_page_index() {
        let mut map = MessagesByMonth::new();
        map.insert(2019, BTreeSet::new());
        map.insert(2020, BTreeSet::new());

        let page = create_index_page(&map);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_page_year() {
        let mut set = BTreeSet::new();
        set.insert(get_message());
        set.insert(get_another_message());
        let page = create_list_page(2020, set);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_lang() {
        assert_eq!(get_message().lang(), "en");
        assert_eq!(get_another_message().lang(), "it");
        let text = "洪沛东谢家霖自习课经常说话，纪律委员金涵笑大怒";
        println!("{:?}", detect_lang(text));

        let two = isolang::Language::from_639_3("cmn").unwrap().to_639_1();
        println!("{:?}", two);


    }

    fn to_data_url<T: AsRef<[u8]>>(input: T, content_type: &str) -> String {
        let base64 = base64::encode(input.as_ref());
        format!("data:{};base64,{}", content_type, base64)
    }

    fn get_message() -> Message {
        Message {
            msg: "Atoms are made of universes".to_string(),
            date: NaiveDateTime::from_timestamp(1445192722 as i64, 0),
            txid: Txid::default(),
        }
    }
    fn get_another_message() -> Message {
        Message {
            msg: "Ciao mi chiamo Gianni e sono italiano".to_string(),
            date: NaiveDateTime::from_timestamp(1445194722 as i64, 0),
            txid: Txid::default(),
        }
    }
}
