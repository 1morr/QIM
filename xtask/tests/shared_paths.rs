//! `qim-databuild` 寫、`qim-bench` 讀的路徑與檔頭在兩個 crate 各寫一份（plan §1.3 不讓
//! `qim-bench` 依賴 `qim-databuild`）；`xtask` 不受依賴規則限制，在這裡比對兩邊。

use std::path::Path;

use qim_databuild::corpora::{self, CORPORA, Group};
use qim_databuild::personal;

/// 一邊寫的路徑與檔頭，每項是「項目、值」；兩邊的項目依同樣的順序排。
type Shared = Vec<(String, String)>;

/// `qim-databuild` 那邊；dev 題的路徑依 `corpora` 的語料名算。
fn databuild(corpora: &[&str]) -> Shared {
    let mut shared: Shared = vec![
        ("個人集題目".into(), personal::ITEMS.into()),
        ("個人集來源".into(), personal::USERDB.into()),
        ("個人集檔頭".into(), personal::SOURCE_HEADER.into()),
    ];
    shared.extend(
        Group::ALL
            .iter()
            .enumerate()
            .map(|(i, group)| (format!("第 {i} 個基準目錄"), group.dir().into())),
    );
    shared.extend(corpora.iter().map(|name| {
        // 照 `/` 接，Windows 的 `\` 才不算不一致。
        let path = corpora::dev_items_path(Path::new(""), name);
        let parts: Vec<&str> = path
            .components()
            .map(|c| c.as_os_str().to_str().expect("路徑是 UTF-8"))
            .collect();
        (format!("{name} 的 dev 題"), parts.join("/"))
    }));
    shared
}

/// `qim-bench` 那邊，項目與順序同 [`databuild`]。
fn bench(corpora: &[&str]) -> Shared {
    let mut shared: Shared = vec![
        ("個人集題目".into(), qim_bench::PERSONAL.into()),
        ("個人集來源".into(), qim_bench::PERSONAL_USERDB.into()),
        (
            "個人集檔頭".into(),
            qim_bench::PERSONAL_SOURCE_HEADER.into(),
        ),
    ];
    shared.extend(
        qim_bench::CORPUS_DIRS
            .iter()
            .enumerate()
            .map(|(i, dir)| (format!("第 {i} 個基準目錄"), (*dir).into())),
    );
    shared.extend(
        corpora
            .iter()
            .map(|name| (format!("{name} 的 dev 題"), qim_bench::dev_items(name))),
    );
    shared
}

/// 兩邊不一致的項目，一項一行；項目數不同也算。
fn mismatches(databuild: &Shared, bench: &Shared) -> Vec<String> {
    let mut found: Vec<String> = databuild
        .iter()
        .zip(bench)
        .filter(|(d, b)| d != b)
        .map(|((item, d), (_, b))| format!("{item}：qim-databuild {d:?}，qim-bench {b:?}"))
        .collect();
    if databuild.len() != bench.len() {
        found.push(format!(
            "項目數：qim-databuild {}，qim-bench {}",
            databuild.len(),
            bench.len()
        ));
    }
    found
}

fn corpus_names() -> Vec<&'static str> {
    CORPORA.iter().map(|c| c.name).collect()
}

#[test]
fn databuild_and_bench_agree_on_the_shared_paths() {
    let names = corpus_names();
    assert_eq!(
        mismatches(&databuild(&names), &bench(&names)),
        Vec::<String>::new()
    );
}

/// 變異：任一邊的任一項改掉都報出來，而且只報那一項。
#[test]
fn a_change_on_either_side_is_reported() {
    let names = corpus_names();
    let (d, b) = (databuild(&names), bench(&names));
    for i in 0..d.len() {
        let mut changed = d.clone();
        changed[i].1.push('x');
        let found = mismatches(&changed, &b);
        assert_eq!(found.len(), 1, "改 qim-databuild 的 {}：{found:?}", d[i].0);
        assert!(found[0].starts_with(&d[i].0), "{found:?}");

        let mut changed = b.clone();
        changed[i].1.push('x');
        let found = mismatches(&d, &changed);
        assert_eq!(found.len(), 1, "改 qim-bench 的 {}：{found:?}", b[i].0);
    }
    let mut extra = b.clone();
    extra.push(("第 2 個基準目錄".into(), "bench/other".into()));
    assert_eq!(mismatches(&d, &extra).len(), 1);
}

/// 變異：語料改名或加語料時兩邊都照名字算 dev 題的路徑，不算不一致。
#[test]
fn renaming_or_adding_a_corpus_is_not_a_mismatch() {
    let renamed = ["renamed-corpus", "another-corpus"];
    assert_eq!(
        mismatches(&databuild(&renamed), &bench(&renamed)),
        Vec::<String>::new()
    );
}
