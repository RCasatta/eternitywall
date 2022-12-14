use crate::message::Message;
use crate::{now, MessagesByCat};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::collections::BTreeSet;

const NBSP: PreEscaped<&str> = PreEscaped("&nbsp;");

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
        p { (NBSP) }
        footer {
            p { a href="/" { "Home" } " | " a href="/about" { "About" } " | " a href="/contact" { "Contact" }  }
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
                (NBSP)
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

    let blocks_iterator = html! {
        a href="https://github.com/RCasatta/blocks_iterator" { "blocks iterator" }
    };

    let content = html! {
        p { "EternityWall shows message in the Bitcoin blockchain. Due to economic incentives, the Bitcoin blockchain is the nearest thing to digital eternity." }
        p { "A message is a transaction with an OP_RETURN output containing valid utf-8 starting with characters \"EW\"." }
        p { "All dates are referred to the block timestamp containing the transaction and are in UTC." }
        p { "How to " (core_link) " with Bitcoin Core." }
        p { "You can " (electrum_link) ", but remember to start with hex `4557` (EW)."}
        p { "Built with " (blocks_iterator) "." }
    };

    page(content).into_string()
}

pub fn create_contact() -> String {
    let content = html! {
        h2 { "Contact" }
        form action="https://pay2.email" method="POST" {
            label {
                p { "Your email:"}
                input type="email" name="reply_to" { }
            }
            p { (NBSP) }
            label {
                p { "Your message:"}
                textarea name="message" rows="4" cols="50" { }
            }
            input type="hidden" name="to_enc" value="e1v9nk2tt9de3hy7tsw35k7m3wdaexwtmkxy9z603qtqer2df38ys9vaz5d4ukxvz20py5c6rw2fuhqm3jxft57n6pv9zygd26g935jdzrfa38wvfkf344zvt0pfchsjr30ff9ydm00f895ctx9ddx56m2gg6z7dtcwp457krwt9ryjume2ymku63kvgu5gmc295lzqmpkdc4njffdvaex2ctnv5s8qam4pf3y52ehf3hkwcjcf3zy22mvgg6xyan2f9x4wn2vxfqhjmmsdpuyxwrwd9zxw4ntdech24nj29v4gc6x0ftn2afe29jhs666v4456m63pfcx54jstyu9j5nkdeg4qa2sx4ry2ef4xpmkjk23g5ch5e6gwfxysstegy9z6tfdypt8zj6ed56xg52ew4t9qde5w46xsn6p2cuk64tk9dyy7ef5d5mz7upjfpu4ze28t92kg3g2kfezjm3tna7gpsxvrkyykay9war4l99pvzcrm9hhwecsmmz63kr0t7at5ku0mvhlnhckshrsg0kgggeymg4l64z5rja2qgyvn5u" { }
            input type="hidden" name="subject_enc" value="e1v9nk2tt9de3hy7tsw35k7m3wdaexwtmkxy9z603qtqer2df38ysx6vnvgd4yzm3c2adyjajpgetnjee0d3xnv72fgs68wj2kduuhzwpjw32xgk24wemj75tnpfnysu2yxa5hgans24rk5nz6xumnqvmztfk82c6kf3vk6a6009xrzd2s892rg2enwce9z6c295lzq7rey4mj62t7xckkwun9v9ek2zjcda6n2mrvdfp4v7p5xam8xnrng34yw3t5xg6y6jenwekkxd6dge2nyjrydpfy7m6vgsm9gntvt9gk23fk8p4y7vrwvey5jajk2yenvzj3092zk62rvvmnwkfnt95nsvettftyc5zsg9uyy5nswd99wtm9g3dxz3zp2cm8s6j32dhrqmnn2pp9x7z02fe4x4espgkj6tfq0furxmrf2frhwt6cd3yyxt69vfn5j2mhtfy5kn2xvguyycm6295x2jmvx46825ntv9ax7zn48cxpar7u2433vswnlj0r23n3y33gn6d4esc67zj2adrf8ptwttcuqayvzmpvpef9q56gm7n5rpgr5kw99uy6nzws" { }
            p { (NBSP) }
            button type="submit" { "Pay 20 satoshi ⚡ to send" }
            p { (NBSP) }
        }
    };

    page(content).into_string()
}

pub fn create_list_page(title: &str, messages: BTreeSet<Message>) -> String {
    let list = html! {
        h2 { (title) }
        p { (NBSP) }
        ul {
            @for msg in &messages {
                li {
                    p {
                        a href=(msg.link()) { (msg.date()) }
                        " - "
                        { (msg.msg) }
                    }
                }
            }
        }
    };

    page(list).into_string()
}

pub fn create_detail_page(msg: &Message) -> String {
    let link = format!("https://fbbe.info/t/{}", msg.txid);
    let content = html! {
        h2 { (msg.date()) " UTC" }
        h1 { (msg.msg) }
        (NBSP)

        p { a href=(link) { "View tx" } }
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
    fn to_data_url<T: AsRef<[u8]>>(input: T, content_type: &str) -> String {
        let base64 = base64::encode(input.as_ref());
        format!("data:{};base64,{}", content_type, base64)
    }
}
