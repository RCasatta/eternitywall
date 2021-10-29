use crate::message::Message;
use crate::{now, MessagesByCat};
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
        p { "&nbsp;" }
        footer {
            p { a href="/" { "Home" } " | " a href="/about" { "About" } " | " a href="/language" { "By language" } " | " a href="/contact" { "Contact" }  }
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
            body style="font-family: Arial, Helvetica, sans-serif;" {
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
        p { "&nbsp;" }
        ul {
            @for cat in cats {
                li {
                    p {
                        a href=(link_cat(cat)) { (cat) }
                        " ("
                        (map.get(cat).unwrap().len().to_string())
                        ")"
                    }
                }
            }
        }
    };

    page(list).into_string()
}

pub fn create_about() -> String {
    let core_link = html! {
        a href="https://blog.eternitywall.com/2016/06/01/how-to-write-a-message-on-the-blockchain" { "write a message" }
    };

    let electrum_link = html! {
        a href="https://twitter.com/electrumwallet/status/1380058456854528001" { "use electrum too" }
    };

    let content = html! {
        p { "EternityWall shows message in the Bitcoin blockchain. Due to economic incentives, the Bitcoin blockchain is the nearest thing to digital eternity." }
        p { "A message is a transaction with an OP_RETURN output containing valid utf-8 starting with characters \"EW\"." }
        p { "All dates are referred to the block timestamp containing the transaction and are in UTC." }
        p { "Languages are automatically detected and they may be wrong." }
        p { "How to " (core_link) " with Bitcoin Core" }
        p { "You can " (electrum_link) ", but remember to start with hex `4557` (EW) "}
    };

    page(content).into_string()
}

pub fn create_contact() -> String {
    let content = html! {
        h2 { "Contact" }
        form action="https://formspree.io/f/xnqlrbey" method="POST" {
            label {
                p { "Your email:"}
                input type="email" name="_replyto" { }
            }
            p { "&nbsp;" }
            label {
                p { "Your message:"}
                textarea name="message" rows="4" cols="50" { }
            }
            input type="hidden" name="_tags" value="eternitywall.it" { }
            p { "&nbsp;" }
            button type="submit" { "Send" }
            p { "&nbsp;" }
        }
    };

    page(content).into_string()
}

pub fn create_list_page(title: &str, messages: BTreeSet<Message>) -> String {
    let list = html! {
        h2 { (title) }
        p { "&nbsp;" }
        ul {
            @for msg in &messages {
                @if let Some(lang) = msg.lang() {
                    li {
                        p {
                            a href=(msg.link()) { (msg.date()) }
                            " - "
                            span lang=(lang) { (msg.msg) }
                        }
                    }
                } @else {
                    li {
                        p {
                            a href=(msg.link()) { (msg.date()) }
                            " - "
                            { (msg.msg) }
                        }
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
            h1 { span lang=(lang) { (msg.msg) }  }
        } @else {
            h1 { (msg.msg) }
        }

    };

    page(content).into_string()
}

fn link_cat(cat: &str) -> String {
    format!("/{}", cat)
}

#[cfg(test)]
mod test {
    use crate::message::test::{get_another_message, get_message};
    use crate::templates::{create_detail_page, create_index_page, create_list_page, page};
    use crate::MessagesByCat;
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
    fn test_escape() {
        let a = html! { p { "<>" } };
        assert_eq!(a.into_string(), "<p>&lt;&gt;</p>");
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
        let mut map = MessagesByCat::new();
        map.insert("2019".to_string(), BTreeSet::new());
        map.insert("2020".to_string(), BTreeSet::new());

        let page = create_index_page(&map, true);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_page_year() {
        let mut set = BTreeSet::new();
        set.insert(get_message());
        set.insert(get_another_message());
        let page = create_list_page("2020", set);
        assert_eq!("", to_data_url(&page, "text/html"));
    }

    #[test]
    fn test_lang() {
        assert_eq!(get_message().lang(), Some("en"));
        assert_eq!(get_another_message().lang(), Some("it"));
        let text = "洪沛东谢家霖自习课经常说话，纪律委员金涵笑大怒";
        println!("{:?}", detect_lang(text));
    }

    fn to_data_url<T: AsRef<[u8]>>(input: T, content_type: &str) -> String {
        let base64 = base64::encode(input.as_ref());
        format!("data:{};base64,{}", content_type, base64)
    }
}
