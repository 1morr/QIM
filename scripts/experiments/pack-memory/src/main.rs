//! M2 票 04：依序載入六份資料包、一個 `Engine` 與預設佈局，每步記常駐記憶體的增量與耗時。
//! 跑法與指標的意義見同目錄的 README.md。`--features schema` 時佈局走 `qim_layout::validate`
//! （含 JSON Schema），否則只用 `serde_json` 讀進型別。環境變數 `LEXICON` 換詞庫（M3 票 24）。

use std::fs;
use std::hint::black_box;
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use qim_core::{
    Engine, EnglishPack, LanguageModelPack, LexiconPack, Pack, RadicalHintPack, SchemePack,
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
    /// `malloc_zone_statistics(NULL)` 的 `size_in_use`：所有 zone 合計還沒釋放的配置。
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
    name: String,
    file: Option<u64>,
    before: Sample,
    after: Sample,
    elapsed: Duration,
}

fn mb(bytes: u64) -> f64 {
    bytes as f64 / MB
}

fn delta_mb(after: u64, before: u64) -> f64 {
    (after as f64 - before as f64) / MB
}

/// 量 `step` 前後的差記成一列；`step` 回傳它的結果與讀了多少位元組的檔案。
fn measure<T>(
    rows: &mut Vec<Row>,
    name: impl Into<String>,
    step: impl FnOnce() -> (T, Option<u64>),
) -> T {
    let before = sample();
    let started = Instant::now();
    let (value, file) = step();
    let elapsed = started.elapsed();
    rows.push(Row {
        name: name.into(),
        file,
        before,
        after: sample(),
        elapsed,
    });
    value
}

/// 讀檔、解析、丟掉檔案位元組，量這整段；檔案的緩衝區沒還給系統的部分也算進去。
fn load<T>(
    rows: &mut Vec<Row>,
    name: impl Into<String>,
    path: &Path,
    parse: impl FnOnce(&[u8]) -> T,
) -> T {
    measure(rows, name, || {
        let bytes = read(path);
        let value = black_box(parse(&bytes));
        (value, Some(bytes.len() as u64))
    })
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path)
        .unwrap_or_else(|e| panic!("讀不了 {}：{e}（先跑 cargo xtask data）", path.display()))
}

/// 要 libmalloc 把空出來的頁還給系統，回傳還了多少位元組。
fn relieve() -> u64 {
    // SAFETY: 簽章照 `malloc/malloc.h` 的 `size_t malloc_zone_pressure_relief(malloc_zone_t *zone,
    // size_t goal)`；只歸還空頁，不碰還在用的配置。
    unsafe { malloc_zone_pressure_relief(std::ptr::null_mut(), 0) as u64 }
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

/// `PROBE=hashset`：在沒有別的垃圾頁的新程序裡，量 `Engine::new` 為詞頻總和建的那種 HashSet。
/// 檔案位元組留到最後才丟，免得 HashSet 重用它釋放的頁。
fn hashset_probe(build: &Path) {
    let bytes = read(&pack(build, &lexicon_id()));
    let words = LexiconPack::from_bytes(&bytes).unwrap().lexicon;
    let before = sample();
    let texts = words
        .iter()
        .flat_map(|(_, w)| w.iter().map(|w| w.text.as_str()))
        .collect::<std::collections::HashSet<_>>()
        .len();
    let after = sample();
    black_box((&bytes, &words));
    println!(
        "PROBE=hashset：新程序裡對 {texts} 個詞建 HashSet<&str> 再丟掉後 footprint 增加 {:.2} MB、heap 增加 {:.2} MB",
        delta_mb(after.footprint, before.footprint),
        delta_mb(after.heap, before.heap)
    );
}

/// 要量的詞庫：`LEXICON` 環境變數，預設方案指名的 `quick-lexicon`。
fn lexicon_id() -> String {
    std::env::var("LEXICON").unwrap_or_else(|_| "quick-lexicon".into())
}

fn main() {
    let lexicon_id = lexicon_id();
    let build = std::env::args().nth(1).map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../build"),
        PathBuf::from,
    );
    if std::env::var_os("PROBE").is_some_and(|p| p == "hashset") {
        return hashset_probe(&build);
    }
    let start = sample();
    let mut rows = Vec::new();

    let scheme = load(&mut rows, "速成 `quick`", &pack(&build, "quick"), |b| {
        SchemePack::from_bytes(b).unwrap()
    });
    let lexicon = load(
        &mut rows,
        format!("詞庫 `{lexicon_id}`"),
        &pack(&build, &lexicon_id),
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
    let engine = measure(&mut rows, "`Engine`（拿走上面五份）", || {
        let engine = Engine::new(scheme, Some(lexicon), Some(bigram))
            .unwrap()
            .with_simplifier(simplifier)
            .with_english(english);
        (engine, None)
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
    let relieved = relieve();
    let relief = sample();
    // 以下都在量完之後，暫存的配置不混進上面的增量。
    // 另讀一次詞庫，數結構；同 `Engine::new` 算詞頻總和，對全部詞建一個 HashSet 再丟掉。
    let words = LexiconPack::from_bytes(&read(&pack(&build, &lexicon_id)))
        .unwrap()
        .lexicon;
    let word_codes = words.iter().count();
    let word_entries: usize = words.iter().map(|(_, w)| w.len()).sum();
    let before_set = sample();
    let word_texts = words
        .iter()
        .flat_map(|(_, w)| w.iter().map(|w| w.text.as_str()))
        .collect::<std::collections::HashSet<_>>()
        .len();
    let after_set = sample();
    // 對照：配置 13 MB、寫滿、丟掉，看釋放的大塊配置會不會從 footprint 退掉，再要 libmalloc 還一次。
    let probe = vec![1u8; 13 << 20];
    black_box(&probe);
    drop(probe);
    let freed = sample();
    let relieved_again = relieve();
    let relief_again = sample();
    let candidates = engine.candidates(&UserModel::default(), "anr");
    assert!(candidates.iter().any(|c| c == "門口"), "anr 沒有「門口」");
    black_box((&engine, &hint, &layout));

    let exe = std::env::current_exe().unwrap();
    println!(
        "schema={} 執行檔 {:.2} MB（{}）",
        cfg!(feature = "schema"),
        mb(fs::metadata(&exe).unwrap().len()),
        exe.display()
    );
    println!(
        "開始時 RSS {:.2} MB、footprint {:.2} MB、heap {:.2} MB",
        mb(start.rss),
        mb(start.footprint),
        mb(start.heap)
    );
    println!();
    println!("| 步驟 | 檔案 | RSS 增加 | footprint 增加 | heap 增加 | footprint ÷ 檔案 | 耗時 |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: | ---: |");
    for row in &rows {
        let footprint = delta_mb(row.after.footprint, row.before.footprint);
        let (file, ratio) = match row.file {
            Some(bytes) => {
                let file = mb(bytes);
                (format!("{file:.2} MB"), format!("{:.1}×", footprint / file))
            }
            None => ("—".into(), "—".into()),
        };
        println!(
            "| {} | {file} | {:.2} MB | {footprint:.2} MB | {:.2} MB | {ratio} | {:.1} ms |",
            row.name,
            delta_mb(row.after.rss, row.before.rss),
            delta_mb(row.after.heap, row.before.heap),
            row.elapsed.as_secs_f64() * 1000.0,
        );
    }
    let elapsed: Duration = rows.iter().map(|r| r.elapsed).sum();
    let files: u64 = rows.iter().filter_map(|r| r.file).sum();
    let footprint = delta_mb(end.footprint, start.footprint);
    println!(
        "| 合計 | {:.2} MB | {:.2} MB | {footprint:.2} MB | {:.2} MB | {:.1}× | {:.1} ms |",
        mb(files),
        delta_mb(end.rss, start.rss),
        delta_mb(end.heap, start.heap),
        footprint / mb(files),
        elapsed.as_secs_f64() * 1000.0,
    );
    println!();
    println!(
        "結束時 RSS {:.2} MB、footprint {:.2} MB、heap {:.2} MB；footprint 最高 {:.2} MB",
        mb(end.rss),
        mb(end.footprint),
        mb(end.heap),
        mb(end.peak_footprint)
    );
    println!(
        "malloc_zone_pressure_relief 還給系統 {:.2} MB；之後 RSS {:.2} MB、footprint {:.2} MB",
        mb(relieved),
        mb(relief.rss),
        mb(relief.footprint)
    );
    println!(
        "對照：對 {word_texts} 個詞建 HashSet<&str> 再丟掉後 footprint 增加 {:.2} MB、heap 增加 {:.2} MB",
        delta_mb(after_set.footprint, before_set.footprint),
        delta_mb(after_set.heap, before_set.heap)
    );
    println!(
        "對照：配置 13 MB 寫滿再丟掉後 footprint 增加 {:.2} MB、heap 增加 {:.2} MB；再呼叫 malloc_zone_pressure_relief 還給系統 {:.2} MB，footprint 變化 {:.2} MB",
        delta_mb(freed.footprint, after_set.footprint),
        delta_mb(freed.heap, after_set.heap),
        mb(relieved_again),
        delta_mb(relief_again.footprint, freed.footprint)
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
