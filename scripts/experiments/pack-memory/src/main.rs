//! M2 票 04：依序載入六份資料包、一個 `Engine` 與預設佈局，每步記常駐記憶體的增量。
//! 跑法與指標的意義見同目錄的 README.md。`--features schema` 時佈局走 `qim_layout::validate`
//! （含 JSON Schema），否則只用 `serde_json` 讀進型別。

use std::fs;
use std::hint::black_box;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};

use qim_core::{
    Engine, EnglishPack, LanguageModelPack, LexiconPack, RadicalHintPack, SchemePack,
    SimplifierPack, UserModel,
};
use qim_layout::Layout;

const MB: f64 = 1_048_576.0;

unsafe extern "C" {
    /// libmalloc（`malloc/malloc.h`）：把 zone 裡空出來的頁還給系統，回傳還了多少位元組；
    /// zone 傳 null 表示所有 zone、goal 傳 0 表示能還多少還多少。libc crate 沒有宣告它。
    fn malloc_zone_pressure_relief(zone: *mut libc::malloc_zone_t, goal: usize) -> usize;
}

/// 一個時間點的讀數，單位位元組。
#[derive(Clone, Copy)]
struct Sample {
    /// `ri_resident_size`：常駐的頁，含共享與檔案映射的頁。
    rss: u64,
    /// `ri_phys_footprint`：系統記憶體上限看的值（README〈指標〉）。
    footprint: u64,
    /// `malloc_zone_statistics` 的 `size_in_use`：預設 zone 裡還沒釋放的配置。
    heap: u64,
    /// `ri_lifetime_max_phys_footprint`：到目前為止 footprint 的最高點。
    peak_footprint: u64,
}

fn sample() -> Sample {
    let mut info = MaybeUninit::<libc::rusage_info_v4>::zeroed();
    // SAFETY: V4 的緩衝區就是 rusage_info_v4；回傳 0 時已填好。
    let info = unsafe {
        let ret = libc::proc_pid_rusage(
            libc::getpid(),
            libc::RUSAGE_INFO_V4,
            info.as_mut_ptr().cast::<libc::rusage_info_t>(),
        );
        assert_eq!(ret, 0, "proc_pid_rusage 失敗");
        info.assume_init()
    };
    let mut stats = MaybeUninit::<libc::malloc_statistics_t>::zeroed();
    // SAFETY: zone 傳 null 表示所有 zone 的合計；函式只寫 stats。
    let stats = unsafe {
        libc::malloc_zone_statistics(std::ptr::null_mut(), stats.as_mut_ptr());
        stats.assume_init()
    };
    Sample {
        rss: info.ri_resident_size,
        footprint: info.ri_phys_footprint,
        heap: stats.size_in_use as u64,
        peak_footprint: info.ri_lifetime_max_phys_footprint,
    }
}

struct Row {
    name: &'static str,
    file: Option<u64>,
    before: Sample,
    after: Sample,
}

fn delta_mb(after: u64, before: u64) -> f64 {
    (after as f64 - before as f64) / MB
}

/// 讀檔、解析、丟掉檔案位元組，量這整段前後的差；檔案的緩衝區沒還給系統的部分也算進去。
fn load<T>(
    rows: &mut Vec<Row>,
    name: &'static str,
    path: &Path,
    parse: impl FnOnce(&[u8]) -> T,
) -> T {
    let before = sample();
    let bytes = fs::read(path)
        .unwrap_or_else(|e| panic!("讀不了 {}：{e}（先跑 cargo xtask data）", path.display()));
    let value = black_box(parse(&bytes));
    let file = bytes.len() as u64;
    drop(bytes);
    rows.push(Row {
        name,
        file: Some(file),
        before,
        after: sample(),
    });
    value
}

fn pack(build: &Path, id: &str) -> PathBuf {
    build.join("packs").join(id).join(format!("{id}.pack"))
}

#[cfg(feature = "schema")]
fn parse_layout(text: &str) -> Layout {
    qim_layout::validate(text, qim_layout::Target::default())
        .into_layout()
        .expect("內建佈局驗得過")
        .0
}

#[cfg(not(feature = "schema"))]
fn parse_layout(text: &str) -> Layout {
    serde_json::from_str(text).expect("內建佈局讀得進型別")
}

fn main() {
    let build = std::env::args().nth(1).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build"),
        PathBuf::from,
    );
    let start = sample();
    let mut rows = Vec::new();

    let scheme = load(&mut rows, "速成 `quick`", &pack(&build, "quick"), |b| {
        SchemePack::from_bytes(b).unwrap()
    });
    let lexicon = load(
        &mut rows,
        "詞庫 `quick-lexicon`",
        &pack(&build, "quick-lexicon"),
        |b| LexiconPack::from_bytes(b).unwrap(),
    );
    let bigram = load(
        &mut rows,
        "bigram `zh-hant-bigram`",
        &pack(&build, "zh-hant-bigram"),
        |b| LanguageModelPack::from_bytes(b).unwrap(),
    );
    let hint = load(
        &mut rows,
        "字根提示 `cangjie-hint`",
        &pack(&build, "cangjie-hint"),
        |b| RadicalHintPack::from_bytes(b).unwrap(),
    );
    let simplifier = load(
        &mut rows,
        "簡繁 `opencc-hk2s`",
        &pack(&build, "opencc-hk2s"),
        |b| SimplifierPack::from_bytes(b).unwrap(),
    );
    let english = load(
        &mut rows,
        "英文 `scowl-en-us`",
        &pack(&build, "scowl-en-us"),
        |b| EnglishPack::from_bytes(b).unwrap(),
    );

    // Engine 拿走速成、詞庫、bigram、簡繁、英文；這一步量的是它另外建的（依字頻重排的碼表等）。
    let before = sample();
    let engine = Engine::new(scheme, Some(lexicon), Some(bigram))
        .unwrap()
        .with_simplifier(simplifier)
        .with_english(english);
    rows.push(Row {
        name: "`Engine`（拿走上面五份）",
        file: None,
        before,
        after: sample(),
    });

    let layout_path = build.join("layouts/gboard-quick.json");
    let layout_name = if cfg!(feature = "schema") {
        "佈局 `gboard-quick.json`（Schema 驗證）"
    } else {
        "佈局 `gboard-quick.json`（不驗證）"
    };
    let layout = load(&mut rows, layout_name, &layout_path, |b| {
        parse_layout(std::str::from_utf8(b).unwrap())
    });

    let end = sample();
    // SAFETY: 只要求 libmalloc 歸還空頁，不碰還在用的配置。
    let relieved = unsafe { malloc_zone_pressure_relief(std::ptr::null_mut(), 0) };
    let relief = sample();
    // 對照：配置 13 MB、寫滿、丟掉，看釋放的大塊配置會不會從 footprint 退掉。
    let probe = vec![1u8; 13 << 20];
    black_box(&probe);
    drop(probe);
    let freed = sample();
    // 量完才另外讀一次詞庫數結構，免得暫存的配置混進上面的增量。
    let words = LexiconPack::from_bytes(&fs::read(pack(&build, "quick-lexicon")).unwrap())
        .unwrap()
        .lexicon;
    let word_codes = words.iter().count();
    let word_entries: usize = words.iter().map(|(_, w)| w.len()).sum();
    let word_texts = words
        .iter()
        .flat_map(|(_, w)| w.iter().map(|w| w.text.as_str()))
        .collect::<std::collections::HashSet<_>>()
        .len();
    let candidates = engine.candidates(&UserModel::default(), "anr");
    assert!(candidates.iter().any(|c| c == "門口"), "anr 沒有「門口」");
    black_box((&engine, &hint, &layout));

    let exe = std::env::current_exe().unwrap();
    println!(
        "schema={} 執行檔 {:.2} MB（{}）",
        cfg!(feature = "schema"),
        fs::metadata(&exe).unwrap().len() as f64 / MB,
        exe.display()
    );
    println!(
        "開始時 RSS {:.2} MB、footprint {:.2} MB、heap {:.2} MB",
        start.rss as f64 / MB,
        start.footprint as f64 / MB,
        start.heap as f64 / MB
    );
    println!();
    println!("| 步驟 | 檔案 | RSS 增加 | footprint 增加 | heap 增加 | footprint ÷ 檔案 |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: |");
    for row in &rows {
        let footprint = delta_mb(row.after.footprint, row.before.footprint);
        let (file, ratio) = match row.file {
            Some(bytes) => {
                let mb = bytes as f64 / MB;
                (format!("{mb:.2} MB"), format!("{:.1}×", footprint / mb))
            }
            None => ("—".into(), "—".into()),
        };
        println!(
            "| {} | {file} | {:.2} MB | {footprint:.2} MB | {:.2} MB | {ratio} |",
            row.name,
            delta_mb(row.after.rss, row.before.rss),
            delta_mb(row.after.heap, row.before.heap),
        );
    }
    let files: u64 = rows.iter().filter_map(|r| r.file).sum();
    let footprint = delta_mb(end.footprint, start.footprint);
    println!(
        "| 合計 | {:.2} MB | {:.2} MB | {footprint:.2} MB | {:.2} MB | {:.1}× |",
        files as f64 / MB,
        delta_mb(end.rss, start.rss),
        delta_mb(end.heap, start.heap),
        footprint / (files as f64 / MB),
    );
    println!();
    println!(
        "結束時 RSS {:.2} MB、footprint {:.2} MB、heap {:.2} MB；footprint 最高 {:.2} MB",
        end.rss as f64 / MB,
        end.footprint as f64 / MB,
        end.heap as f64 / MB,
        end.peak_footprint as f64 / MB
    );
    println!(
        "malloc_zone_pressure_relief 還給系統 {:.2} MB；之後 RSS {:.2} MB、footprint {:.2} MB",
        relieved as f64 / MB,
        relief.rss as f64 / MB,
        relief.footprint as f64 / MB
    );
    println!(
        "對照：配置 13 MB 寫滿再丟掉後 footprint 增加 {:.2} MB、heap 增加 {:.2} MB",
        delta_mb(freed.footprint, relief.footprint),
        delta_mb(freed.heap, relief.heap)
    );
    println!(
        "詞庫 {} 個碼、{} 個詞條（不重複的詞 {} 個）",
        word_codes, word_entries, word_texts
    );
    println!(
        "anr 前三個候選：{}",
        candidates[..3.min(candidates.len())].join("、")
    );
}
