//! 英文模式每鍵延遲（M4 票 25）：逐字打一段含錯字的英文，量 `Session::handle` 的時間。三種設定：只補完
//! （英文容錯關，等於票 25 之前）、英文容錯開（預設）、再開自動修正。用法：
//! `cargo run --release -- <repo>/build/packs`。

use qim_core::{
    Engine, EnglishPack, Key, LanguageModelPack, LexiconPack, Options, Pack, SchemePack, Session,
    UserModel,
};
use std::time::Instant;

const TEXT: &str = "my freind said teh goverment is wierd and i recieve the definately best piece of \
knowlege truely sucess finaly adress suprise libary existance posession peice neccessary gogle acheive \
becuase hello world ivan.test@example.com qimzorb begining embarass occured calender ";

fn main() {
    let root = std::env::args().nth(1).expect("用法：autocorrect-latency <build/packs>");
    let read = |id: &str| std::fs::read(format!("{root}/{id}/{id}.pack")).expect("讀資料包");
    let engine = Engine::new(
        SchemePack::from_bytes(&read("quick")).unwrap(),
        Some(LexiconPack::from_bytes(&read("quick-lexicon")).unwrap()),
        Some(LanguageModelPack::from_bytes(&read("zh-hant-bigram")).unwrap()),
    )
    .unwrap()
    .with_english(EnglishPack::from_bytes(&read("aosp-en-us")).unwrap());
    let settings = [
        ("只補完", false, false),
        ("英文容錯", true, false),
        ("英文容錯＋自動修正", true, true),
    ];
    println!("| 設定 | 鍵數 | p50（µs） | p95（µs） | 最大（µs） |\n| --- | ---: | ---: | ---: | ---: |");
    for (name, english_corrections, autocorrect) in settings {
        let mut times = Vec::new();
        for _ in 0..20 {
            let mut user = UserModel::default();
            let mut session = Session::default();
            session.set_options(Options {
                english_corrections,
                autocorrect,
                ..Options::default()
            });
            session.handle(&engine, &mut user, Key::ToggleEnglish);
            for c in TEXT.chars() {
                let key = match c {
                    ' ' => Key::Space,
                    c if c.is_ascii_alphabetic() => Key::Letter(c),
                    c => Key::Punct(c),
                };
                let start = Instant::now();
                std::hint::black_box(session.handle(&engine, &mut user, key));
                times.push(start.elapsed().as_secs_f64() * 1e6);
            }
        }
        times.sort_by(f64::total_cmp);
        let at = |p: f64| times[((times.len() - 1) as f64 * p) as usize];
        println!(
            "| {name} | {} | {:.1} | {:.1} | {:.1} |",
            times.len(),
            at(0.5),
            at(0.95),
            at(1.0)
        );
    }
}
