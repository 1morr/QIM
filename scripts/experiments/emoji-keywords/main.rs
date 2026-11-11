//! 票 16：CLDR 的關鍵字收到多泛才不會在常用詞後面塞不相關的 emoji。每個門檻（關鍵字最多對到
//! 幾個 emoji 才收）建一次 emoji 對照表，用 `build/packs/` 的速成、詞庫、bigram 資料包經 `Session`
//! 打探測詞，印第一頁。`WRITE=<門檻>` 時把那個門檻的 emoji 資料包寫到 `build/packs/`，再跑
//! `cargo xtask bench` 量修正次數。跑法見同目錄的 README.md。只讀快取，不下載。

use std::collections::BTreeMap;
use std::path::Path;

use qim_core::{
    EmojiPack, Engine, Key, LanguageModelPack, LexiconPack, Options, Pack, SchemePack, Session,
    UserModel,
};
use qim_databuild::common_forms::CommonForms;
use qim_databuild::corpora::{Conversion, Converter};
use qim_databuild::emoji::{self, Annotations};
use qim_databuild::sources::{self, Fetch};

struct NoFetch;
impl Fetch for NoFetch {
    fn fetch(&self, url: &str, _: &mut dyn std::io::Write) -> Result<(), String> {
        Err(format!("快取沒有 {url}，先跑 cargo xtask data"))
    }
}

const LIMITS: [usize; 9] = [usize::MAX, 20, 10, 6, 5, 4, 3, 2, 1];
/// 抽查：要有 emoji 的（票 16 的「笑」「貓」「多謝」），與 CLDR 關鍵字太泛、全收時會接上笑臉的（「打開」「眼」「臉」）。
const PROBES: [&str; 14] = [
    "笑", "貓", "多謝", "狗", "心", "哈哈", "開心", "打開", "眼", "臉", "嘴", "手", "日", "動物",
];
const EMOJI_PACK: &str = "build/packs/cldr-emoji-yue-hant/cldr-emoji-yue-hant.pack";

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |rel: &str| std::fs::read(root.join(rel)).unwrap_or_else(|e| panic!("{rel}：{e}"));
    let all = sources::parse(&String::from_utf8(read(sources::SOURCES)).unwrap()).unwrap();
    let cache = sources::default_cache_dir().unwrap().join("sources");
    let load = |id: &str, path: &str| -> String {
        let file = all.source.get(id).unwrap().file(path).unwrap();
        String::from_utf8(sources::load(&cache, id, file, &NoFetch).unwrap()).unwrap()
    };
    let scheme = SchemePack::from_bytes(&read("build/packs/quick/quick.pack")).unwrap();
    let lexicon =
        LexiconPack::from_bytes(&read("build/packs/quick-lexicon/quick-lexicon.pack")).unwrap();
    let model =
        LanguageModelPack::from_bytes(&read("build/packs/zh-hant-bigram/zh-hant-bigram.pack"))
            .unwrap();
    // 每個字取它排得最前的碼。
    let mut code_of: BTreeMap<char, (usize, String)> = BTreeMap::new();
    for (code, entries) in scheme.code_table.iter() {
        for (rank, e) in entries.iter().enumerate() {
            let better = code_of.get(&e.text).is_none_or(|(r, _)| rank < *r);
            if better {
                code_of.insert(e.text, (rank, code.to_string()));
            }
        }
    }
    let converter = Converter::new(
        Conversion::T2hk,
        CommonForms::from_opencc(&scheme.code_table).unwrap(),
    )
    .unwrap();
    let annotations: Vec<(&str, String)> = emoji::CLDR_ANNOTATIONS
        .iter()
        .map(|(name, path)| (*name, load(emoji::CLDR, path)))
        .collect();
    let emoji_test = load(emoji::EMOJI_TEST, emoji::EMOJI_TEST_FILE);
    let base = Engine::new(scheme.clone(), Some(lexicon), Some(model)).unwrap();
    let write: Option<usize> = std::env::var("WRITE").ok().map(|v| v.parse().unwrap());

    for limit in LIMITS {
        let built = emoji::table(
            &annotations
                .iter()
                .map(|(name, xml)| Annotations { name, xml })
                .collect::<Vec<_>>(),
            &emoji_test,
            &|w| converter.convert(w),
            &|c| code_of.contains_key(&c),
            limit,
        )
        .unwrap();
        let shown = if limit == usize::MAX {
            "不限".into()
        } else {
            limit.to_string()
        };
        println!(
            "## 門檻 {shown}：太泛不收的關鍵字 {}，收 {} 個字詞、{} 個 emoji",
            built.counts.broad, built.counts.words, built.counts.emoji
        );
        let pack = EmojiPack {
            id: "cldr-emoji-yue-hant".into(),
            emoji: built.table,
        };
        if write == Some(limit) {
            std::fs::write(root.join(EMOJI_PACK), pack.to_bytes().unwrap()).unwrap();
            println!("（已寫到 {EMOJI_PACK}）");
        }
        let engine = base.clone().with_emoji(pack);
        for word in PROBES {
            let code: String = word.chars().map(|c| code_of[&c].1.as_str()).collect();
            let page = first_page(&engine, &code);
            let at = page.iter().position(|t| t == word);
            let marked: Vec<String> = page
                .iter()
                .map(|t| {
                    if t == word {
                        format!("[{t}]")
                    } else {
                        t.clone()
                    }
                })
                .collect();
            let note = if at.is_none() {
                "（不在第一頁）"
            } else {
                ""
            };
            println!("- {word} `{code}`{note}：{}", marked.join(" "));
        }
        println!();
    }
}

fn first_page(engine: &Engine, code: &str) -> Vec<String> {
    let mut session = Session::default();
    session.set_options(Options {
        incognito: true,
        ..Options::default()
    });
    let mut user = UserModel::default();
    let mut candidates = Vec::new();
    for c in code.chars() {
        candidates = session.handle(engine, &mut user, Key::Letter(c)).candidates;
    }
    candidates.into_iter().map(|c| c.text).collect()
}
