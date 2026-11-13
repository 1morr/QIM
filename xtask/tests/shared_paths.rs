//! `qim-databuild` 寫、`qim-bench` 讀的路徑與檔頭在兩個 crate 各寫一份（plan〈依賴規則〉 不讓
//! `qim-bench` 依賴 `qim-databuild`）；`xtask` 不受依賴規則限制，在這裡比對兩邊。

use std::path::Path;

use qim_databuild::corpora::{self, CORPORA, Group};
use qim_databuild::{personal, typo};

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
    shared.extend(
        Group::ALL
            .iter()
            .enumerate()
            .map(|(i, group)| (format!("第 {i} 個有前文目錄"), group.context_dir().into())),
    );
    shared.extend(
        Group::ALL
            .iter()
            .filter_map(|group| group.typo_dir())
            .enumerate()
            .map(|(i, dir)| (format!("第 {i} 個打錯碼目錄"), dir.into())),
    );
    shared.extend(
        Group::ALL
            .iter()
            .filter_map(|group| group.touch_dir())
            .enumerate()
            .map(|(i, dir)| (format!("第 {i} 個觸控模擬目錄"), dir.into())),
    );
    shared.push(("觸控模擬的佈局".into(), typo::TOUCH_LAYOUT.into()));
    shared.push((
        "觸控模擬的畫布尺寸".into(),
        typo::TOUCH_MM_PER_UNIT.to_string(),
    ));
    // 觸控錯誤模型的參數：`qim_bench::Taps` 抽點對鍵的點擊用的那份複本（M4 票 09c）。
    let noise = typo::TouchNoise::default();
    shared.extend([
        ("觸控錯誤模型的 σx".into(), noise.sigma_x_mm.to_string()),
        ("觸控錯誤模型的 σy".into(), noise.sigma_y_mm.to_string()),
        ("觸控錯誤模型的旋轉".into(), noise.rotation_deg.to_string()),
        ("觸控錯誤模型的往下偏".into(), noise.down_mm.to_string()),
        (
            "觸控錯誤模型的往中心拉".into(),
            noise.edge_pull_sigma.to_string(),
        ),
    ]);
    // 照 `/` 接，Windows 的 `\` 才不算不一致。
    let slashed = |path: std::path::PathBuf| {
        let parts: Vec<String> = path
            .components()
            .map(|c| c.as_os_str().to_str().expect("路徑是 UTF-8").to_string())
            .collect();
        parts.join("/")
    };
    for name in corpora {
        let root = Path::new("");
        shared.push((
            format!("{name} 的 dev 題"),
            slashed(corpora::dev_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的 dev 有前文題"),
            slashed(corpora::dev_context_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的長前文題"),
            slashed(corpora::context_long_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的 dev 長前文題"),
            slashed(corpora::dev_context_long_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的口語長前文題"),
            slashed(corpora::dialogue_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的 dev 口語長前文題"),
            slashed(corpora::dev_dialogue_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的 dev 打錯碼題"),
            slashed(corpora::dev_typo_items_path(root, name)),
        ));
        shared.push((
            format!("{name} 的 dev 觸控模擬題"),
            slashed(corpora::dev_touch_items_path(root, name)),
        ));
    }
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
        qim_bench::CONTEXT_DIRS
            .iter()
            .enumerate()
            .map(|(i, dir)| (format!("第 {i} 個有前文目錄"), (*dir).into())),
    );
    shared.extend(
        qim_bench::TYPO_DIRS
            .iter()
            .enumerate()
            .map(|(i, dir)| (format!("第 {i} 個打錯碼目錄"), (*dir).into())),
    );
    shared.extend(
        qim_bench::TOUCH_DIRS
            .iter()
            .enumerate()
            .map(|(i, dir)| (format!("第 {i} 個觸控模擬目錄"), (*dir).into())),
    );
    shared.push(("觸控模擬的佈局".into(), qim_bench::TOUCH_LAYOUT.into()));
    shared.push((
        "觸控模擬的畫布尺寸".into(),
        qim_bench::TOUCH_MM_PER_UNIT.to_string(),
    ));
    shared.extend([
        (
            "觸控錯誤模型的 σx".into(),
            qim_bench::TOUCH_SIGMA_X_MM.to_string(),
        ),
        (
            "觸控錯誤模型的 σy".into(),
            qim_bench::TOUCH_SIGMA_Y_MM.to_string(),
        ),
        (
            "觸控錯誤模型的旋轉".into(),
            qim_bench::TOUCH_ROTATION_DEG.to_string(),
        ),
        (
            "觸控錯誤模型的往下偏".into(),
            qim_bench::TOUCH_DOWN_MM.to_string(),
        ),
        (
            "觸控錯誤模型的往中心拉".into(),
            qim_bench::TOUCH_EDGE_PULL_SIGMA.to_string(),
        ),
    ]);
    for name in corpora {
        shared.push((format!("{name} 的 dev 題"), qim_bench::dev_items(name)));
        shared.push((
            format!("{name} 的 dev 有前文題"),
            qim_bench::dev_context_items(name),
        ));
        shared.push((
            format!("{name} 的長前文題"),
            qim_bench::context_long_items(name),
        ));
        shared.push((
            format!("{name} 的 dev 長前文題"),
            qim_bench::dev_context_long_items(name),
        ));
        shared.push((
            format!("{name} 的口語長前文題"),
            qim_bench::dialogue_items(name),
        ));
        shared.push((
            format!("{name} 的 dev 口語長前文題"),
            qim_bench::dev_dialogue_items(name),
        ));
        shared.push((
            format!("{name} 的 dev 打錯碼題"),
            qim_bench::dev_typo_items(name),
        ));
        shared.push((
            format!("{name} 的 dev 觸控模擬題"),
            qim_bench::dev_touch_items(name),
        ));
    }
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
