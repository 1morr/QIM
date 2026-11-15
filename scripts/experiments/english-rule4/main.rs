//! 中文狀態的英文候選（排位規則 4）在首選字基準的題目上出現幾題（M4 票 22）。
//!
//! 換英文資料包會改變第 2 位被英文搶的次數，`bench --check` 只看首選與修正次數、看不到這個數。
//! 每題用新的 `Session` 與空白的 `UserModel`、預設選項，照基準的鍵打完整題（打錯碼與觸控的題按
//! 實際按的鍵），讀 `Session::inspect` 的候選種類，數第一頁有英文候選的題與英文候選在第 2 位的題。
//!
//! `LIST=1` 另列每個有英文候選的題：公開集合列英文詞與它在資料包的頻率，個人集只列頻率與長度（plan〈首選字基準〉：
//! 個人集的內容不進 repo）。跑法見同目錄的 README。

use std::path::Path;

use qim_bench::{
    ENGLISH_PACK, Personal, load_corpora, load_engine, load_personal, load_typo_sets, physical_key,
};
use qim_core::{CandidateKind, EnglishPack, EnglishWords, Key, Pack, Session, UserModel};

fn main() -> Result<(), String> {
    let root = Path::new(".");
    let list = std::env::var_os("LIST").is_some();
    let mut engine = load_engine(root, &[], None)?.engine;
    // 頻率一律查建好的英文資料包（換成詞表時那份詞表沒有頻率）。
    let built =
        std::fs::read(root.join(ENGLISH_PACK)).map_err(|e| format!("{ENGLISH_PACK}：{e}"))?;
    let built = EnglishPack::from_bytes(&built).map_err(|e| format!("{ENGLISH_PACK}：{e}"))?;
    // 給了詞表就換掉建好的英文資料包：一行一詞，Tab 之後的欄不看。
    if let Some(list) = std::env::args().nth(1) {
        let text = std::fs::read_to_string(&list).map_err(|e| format!("{list}：{e}"))?;
        let words = EnglishWords::new(
            text.lines()
                .map(|l| l.split('\t').next().unwrap_or_default())
                .map(|w| (w, 1)),
        );
        println!("英文詞表：{list}（{} 詞）", words.len());
        engine = engine.with_english(EnglishPack {
            id: "list".into(),
            words,
        });
    } else {
        println!("英文詞表：build/packs 的英文資料包");
    }

    let mut sets = load_corpora(root)?;
    sets.extend(load_typo_sets(root)?);
    if let Personal::Ran(items) = load_personal(root)? {
        sets.push(("personal".into(), items));
    }
    println!();
    println!("| 集合 | 題數 | 有英文候選 | 在第 2 位 |");
    println!("| --- | ---: | ---: | ---: |");
    let (mut total, mut shown, mut second) = (0, 0, 0);
    for (name, items) in &sets {
        let (mut set_shown, mut set_second) = (0, 0);
        for item in items {
            let keys: Vec<Key> = match &item.typed {
                Some(typed) => typed.iter().flatten().cloned().collect(),
                None => item.input().chars().map(physical_key).collect(),
            };
            let mut session = Session::default();
            let mut user = UserModel::default();
            for key in keys {
                session.handle(&engine, &mut user, key);
            }
            let candidates = session.inspect(&engine, &user).candidates;
            if let Some(i) = candidates
                .iter()
                .position(|c| c.kind == CandidateKind::English)
            {
                set_shown += 1;
                set_second += usize::from(i == 1);
                if list {
                    let word = &candidates[i].text;
                    let frequency = built.words.frequency(&word.to_lowercase());
                    if name == "personal" {
                        eprintln!(
                            "{name}\t長度 {}\t頻率 {frequency:?}\t第 {} 位",
                            word.len(),
                            i + 1
                        );
                    } else {
                        eprintln!(
                            "{name}\t第 {} 行\t{word}\t頻率 {frequency:?}\t第 {} 位",
                            item.line,
                            i + 1
                        );
                    }
                }
            }
        }
        println!("| {name} | {} | {set_shown} | {set_second} |", items.len());
        total += items.len();
        shown += set_shown;
        second += set_second;
    }
    println!("| 合計 | {total} | {shown} | {second} |");
    Ok(())
}
